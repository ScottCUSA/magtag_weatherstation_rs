#![no_std]
#![no_main]

use embassy_executor::Spawner;

use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{gpio::Pin, spi::master::AnySpi, timer::timg::TimerGroup};
use esp_println::logger::init_logger_from_env;

use embassy_sync::channel::Channel;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use magtag_weatherstation::weather::model::OpenMeteoResponse;

use crate::tasks::sleep::SleepReason;

pub(crate) static SLEEP_REQUEST: Signal<CriticalSectionRawMutex, (u64, SleepReason)> =
    Signal::new(); // Signal used to notify sleep task of sleep request
pub(crate) static NETWORK_READY: Signal<CriticalSectionRawMutex, ()> = Signal::new(); // Signal used to notify the weather task to begin fetch
pub(crate) static NETWORK_ERROR: Signal<CriticalSectionRawMutex, heapless::String<128>> =
    Signal::new(); // Signal used to notify display task of network/fetch errors
pub(crate) static DATA_CHANNEL: Channel<CriticalSectionRawMutex, OpenMeteoResponse, 1> =
    Channel::new(); // Channel used to deliver weather data to the display task

mod tasks;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize logger
    init_logger_from_env();
    // 64KB heap for network stack, JSON parsing, and HTTP buffers
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);

    log::info!("Initializing peripherals");
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize and start RTOS timer
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    // Spawn the deep sleep task
    // waits for SLEEP_REQUEST signal
    spawner
        .spawn(tasks::sleep::deep_sleep_task(peripherals.LPWR))
        .expect("Failed to spawn deep sleep task");

    // Spawn the display task
    // waits for NETWORK_ERROR signal or DATA_CHANNEL message
    spawner
        .spawn(tasks::display::display_task(
            tasks::display::DisplayResources {
                sclk: peripherals.GPIO36.degrade(),
                mosi: peripherals.GPIO35.degrade(),
                miso: peripherals.GPIO37.degrade(),
                spi2: AnySpi::from(peripherals.SPI2),
                busy: peripherals.GPIO5.degrade(),
                rst: peripherals.GPIO6.degrade(),
                dc: peripherals.GPIO7.degrade(),
                cs: peripherals.GPIO8.degrade(),
            },
        ))
        .expect("Failed to spawn display_task");

    // initialize wifi
    // sends NETWORK_ERROR signal if initializing wifi fails
    let (controller, wifi_interface) =
        match tasks::network::init_radio(tasks::network::RadioResources {
            wifi: peripherals.WIFI,
        }) {
            Ok(pair) => pair,
            Err(e) => {
                log::error!("Failed to initialize radio: {:?}", e);
                loop {
                    // end execution
                    Timer::after(Duration::from_secs(60)).await;
                }
            }
        };
    spawner
        .spawn(tasks::network::wifi_task(controller))
        .expect("Failed to spawn wifi_task");

    // Initialize network stack
    // sends NETWORK_READY signal on link-up and IP acquired
    // sends NETWORK_ERROR signal if link-up time out or acquire IP time out
    let (stack, runner) = tasks::network::init_network_stack(wifi_interface);
    spawner
        .spawn(tasks::network::net_runner_task(runner))
        .expect("Failed to spawn net_runner_task");
    spawner
        .spawn(tasks::network::net_validator_task(stack))
        .expect("Failed to spawn net_validator_task");

    // Spawn the weather fetcher
    // waits for NETWORK_READY signal
    // sends DATA_CHANNEL message on success, NETWORK_ERROR signal on failure
    spawner
        .spawn(tasks::weather::weather_fetcher_task(stack))
        .expect("Failed to spawn weather_fetcher_task");

    // yield to the executor
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}
