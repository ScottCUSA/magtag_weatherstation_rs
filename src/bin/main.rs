#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_time::{Duration, Instant, Timer, with_deadline};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    rng::Rng,
    rtc_cntl::Rtc,
    spi::{self, master::Spi},
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::logger::init_logger;
use esp_radio::Controller;
use heapless::format;
use log::info;
use magtag_weatherstation::{
    display::show_app_error,
    network::{connection, net_task},
    sleep::enter_deep_sleep_secs,
    weather::fetch_and_display_weather,
};

const SLEEP_ON_ERROR_SECS: u64 = 60 * 5;
const SLEEP_ON_SUCCESS_SECS: u64 = 60 * 60 * 24;

const HEAP_KB: usize = 72;

esp_bootloader_esp_idf::esp_app_desc!();

// Use https://docs.rs/static_cell/2.1.1/static_cell/macro.make_static.html
// once rust feature(type_alias_impl_trait) is stable
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize logger for esp-println
    init_logger(log::LevelFilter::Info);
    esp_alloc::heap_allocator!(size: HEAP_KB * 1024);

    info!("Initialize peripherals");
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Initialize RTC for deep sleep
    let rtc = Rtc::new(peripherals.LPWR);

    // Iniitialize SPI device and control pins
    let sclk = peripherals.GPIO36;
    let mosi = peripherals.GPIO35;
    let miso = peripherals.GPIO37;
    let spi = match Spi::new(
        peripherals.SPI2,
        spi::master::Config::default().with_frequency(Rate::from_mhz(4)),
    ) {
        Ok(spi) => spi.with_sck(sclk).with_miso(miso).with_mosi(mosi),
        Err(e) => {
            log::error!("Failed to initialize SPI: {:?}", e);
            enter_deep_sleep_secs(rtc, SLEEP_ON_ERROR_SECS);
        }
    };
    let busy = Input::new(peripherals.GPIO5, InputConfig::default());
    let rst = Output::new(peripherals.GPIO6, Level::Low, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO7, Level::High, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO8, Level::High, OutputConfig::default());
    let spi_device = mk_static!(
        ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
        match ExclusiveDevice::new(spi, cs, Delay::new()) {
            Ok(device) => device,
            Err(e) => {
                log::error!("Failed to create SPI device: {:?}", e);
                enter_deep_sleep_secs(rtc, SLEEP_ON_ERROR_SECS);
            }
        }
    );

    // Initialize and start RTOS timer
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    // Initialize radio and WiFi controller
    let esp_radio_ctrl = &*mk_static!(
        Controller<'static>,
        match esp_radio::init() {
            Ok(ctrl) => ctrl,
            Err(e) => {
                log::error!("Failed to initialize radio: {:?}", e);
                let error_msg: heapless::String<128> =
                    format!("Failed to initialize radio: {e}").unwrap_or_default();
                show_app_error(&error_msg, spi_device, busy, dc, rst);
                enter_deep_sleep_secs(rtc, SLEEP_ON_ERROR_SECS);
            }
        }
    );
    let (controller, interfaces) =
        match esp_radio::wifi::new(esp_radio_ctrl, peripherals.WIFI, Default::default()) {
            Ok(wifi) => wifi,
            Err(e) => {
                log::error!("Failed to initialize WiFi: {:?}", e);
                let error_msg: heapless::String<128> =
                    format!("Failed to initialize WiFi: {e}").unwrap_or_default();
                show_app_error(&error_msg, spi_device, busy, dc, rst);
                enter_deep_sleep_secs(rtc, SLEEP_ON_ERROR_SECS);
            }
        };
    let wifi_interface = interfaces.sta;

    // init network stack
    let config = embassy_net::Config::dhcpv4(Default::default());
    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | (rng.random() as u64);
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    // spawn network tasks
    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();

    // wait for link up (with timeout)
    if with_deadline(Instant::now() + Duration::from_secs(10), async {
        loop {
            if stack.is_link_up() {
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }
    })
    .await
    .is_err()
    {
        log::error!("Timed out waiting for link up");
        show_app_error("Timed out waiting for link up", spi_device, busy, dc, rst);
        enter_deep_sleep_secs(rtc, SLEEP_ON_ERROR_SECS);
    }

    // wait for IP address (with timeout)
    if with_deadline(Instant::now() + Duration::from_secs(20), async {
        loop {
            if let Some(config) = stack.config_v4() {
                info!("Got IP: {}", config.address);
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }
    })
    .await
    .is_err()
    {
        log::error!("Timed out waiting for IP address");
        show_app_error(
            "Timed out waiting for IP address",
            spi_device,
            busy,
            dc,
            rst,
        );
        enter_deep_sleep_secs(rtc, SLEEP_ON_ERROR_SECS);
    }

    let weather_result = fetch_and_display_weather(stack, spi_device, busy, dc, rst).await;

    // Handle result and enter deep sleep
    match weather_result {
        Ok(_) => {
            log::info!("Weather display successful, sleeping for 24 hours");
            enter_deep_sleep_secs(rtc, SLEEP_ON_SUCCESS_SECS);
        }
        Err(_) => {
            log::error!("Fetching weather failed, showing error and sleeping to retry");
            enter_deep_sleep_secs(rtc, SLEEP_ON_ERROR_SECS);
        }
    }
}
