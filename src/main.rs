#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

mod config;
mod display;
mod error;
mod graphics;
mod network;
mod time;
mod weather;

// Use https://docs.rs/static_cell/2.1.1/static_cell/macro.make_static.html
// once rust feature(type_alias_impl_trait) is stable
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write($val);
        x
    }};
}

mod tasks;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::Pin, interrupt::software::SoftwareInterruptControl, spi::master::AnySpi,
    timer::timg::TimerGroup,
};
use esp_println::logger::init_logger_from_env;

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::Channel,
    signal::Signal,
};
use crate::config::SLEEP_ON_ERROR_SECS;
use crate::weather::model::OpenMeteoResponse;

use crate::tasks::sleep::SleepReason;

/// Signal used to notify sleep task of sleep request
pub(crate) static SLEEP_REQUEST: Signal<CriticalSectionRawMutex, (u64, SleepReason)> =
    Signal::new();

/// Signal used to notify the weather task to begin fetch
pub(crate) static NETWORK_READY: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// Signal used to notify display task of network/fetch errors
pub(crate) static NETWORK_ERROR: Signal<CriticalSectionRawMutex, heapless::String<128>> =
    Signal::new();

/// Channel used to deliver weather data to the display task
pub(crate) static DATA_CHANNEL: Channel<CriticalSectionRawMutex, OpenMeteoResponse, 1> =
    Channel::new();

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    init_logger_from_env();
    // 64KB heap for network stack, JSON parsing, and HTTP buffers
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);

    log::info!("Initializing peripherals");
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    spawner.spawn(
        tasks::sleep::deep_sleep_task(peripherals.LPWR).expect("Failed to spawn deep sleep task"),
    );

    spawner.spawn(
        tasks::display::display_task(tasks::display::DisplayResources {
            sclk: peripherals.GPIO36.degrade(),
            mosi: peripherals.GPIO35.degrade(),
            miso: peripherals.GPIO37.degrade(),
            spi2: AnySpi::from(peripherals.SPI2),
            busy: peripherals.GPIO5.degrade(),
            rst: peripherals.GPIO6.degrade(),
            dc: peripherals.GPIO7.degrade(),
            cs: peripherals.GPIO8.degrade(),
        })
        .expect("Failed to spawn display_task"),
    );

    let (controller, wifi_interface) =
        match tasks::network::init_radio(tasks::network::RadioResources {
            wifi: peripherals.WIFI,
        }) {
            Ok(pair) => pair,
            Err(e) => {
                log::error!("Failed to initialize radio: {:?}", e);
                SLEEP_REQUEST.signal((SLEEP_ON_ERROR_SECS, SleepReason::NetworkError));
                loop {
                    Timer::after(Duration::from_secs(60)).await;
                }
            }
        };
    spawner.spawn(tasks::network::wifi_task(controller).expect("Failed to spawn wifi_task"));

    let (stack, runner) = tasks::network::init_network_stack(wifi_interface);
    spawner
        .spawn(tasks::network::net_runner_task(runner).expect("Failed to spawn net_runner_task"));
    spawner.spawn(
        tasks::network::net_validator_task(stack).expect("Failed to spawn net_validator_task"),
    );

    spawner.spawn(
        tasks::weather::weather_fetcher_task(stack).expect("Failed to spawn weather_fetcher_task"),
    );

    loop {
        Timer::after_secs(1).await;
    }
}
