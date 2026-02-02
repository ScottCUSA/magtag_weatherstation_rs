use embassy_futures::select::{Either, select};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig},
    rtc_cntl::Rtc,
    spi::{
        self,
        master::{AnySpi, Spi},
    },
    time::Rate,
};
use magtag_weatherstation::{
    config::{SLEEP_ON_ERROR_SECS, SLEEP_ON_SUCCESS_SECS},
    display::{display_error_text, display_weather},
    mk_static,
    sleep::enter_deep_sleep_secs,
};

use esp_hal::gpio::AnyPin;

use crate::{NETWORK_ERROR, WEATHER_CHANNEL};

pub(crate) struct DisplayResources {
    pub sclk: AnyPin<'static>,
    pub mosi: AnyPin<'static>,
    pub miso: AnyPin<'static>,
    pub spi2: AnySpi<'static>,
    pub busy: AnyPin<'static>,
    pub rst: AnyPin<'static>,
    pub dc: AnyPin<'static>,
    pub cs: AnyPin<'static>,
    pub rtc: &'static mut Rtc<'static>,
}

#[embassy_executor::task]
pub(crate) async fn display_task(resources: DisplayResources) {
    log::info!("Initializing display");
    // Iniitialize SPI device and control pins for display
    let spi = match Spi::new(
        resources.spi2,
        spi::master::Config::default().with_frequency(Rate::from_mhz(4)),
    ) {
        Ok(spi) => spi
            .with_sck(resources.sclk)
            .with_miso(resources.miso)
            .with_mosi(resources.mosi),
        Err(e) => {
            log::error!("Failed to initialize SPI: {:?}", e);
            enter_deep_sleep_secs(resources.rtc, SLEEP_ON_ERROR_SECS);
        }
    };
    let busy = Input::new(resources.busy, InputConfig::default());
    let rst = Output::new(resources.rst, Level::Low, OutputConfig::default());
    let dc = Output::new(resources.dc, Level::High, OutputConfig::default());
    let cs = Output::new(resources.cs, Level::High, OutputConfig::default());

    let spi_device = mk_static!(
        ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
        match ExclusiveDevice::new(spi, cs, Delay::new()) {
            Ok(device) => device,
            Err(e) => {
                log::error!("Failed to create SPI device: {:?}", e);
                enter_deep_sleep_secs(resources.rtc, SLEEP_ON_ERROR_SECS);
            }
        }
    );

    // Wait for either a network error or weather data concurrently.
    match select(NETWORK_ERROR.receive(), WEATHER_CHANNEL.receive()).await {
        Either::First(err_msg) => {
            display_error_text(&err_msg, spi_device, busy, dc, rst);
            enter_deep_sleep_secs(resources.rtc, SLEEP_ON_ERROR_SECS);
        }
        Either::Second(weather_data) => {
            match display_weather(weather_data, spi_device, busy, dc, rst) {
                Ok(_) => {
                    log::info!("Weather display successful, sleeping...");
                    enter_deep_sleep_secs(resources.rtc, SLEEP_ON_SUCCESS_SECS);
                }
                Err(e) => {
                    log::error!("Displaying weather failed: {:?}", e);
                    enter_deep_sleep_secs(resources.rtc, SLEEP_ON_ERROR_SECS);
                }
            }
        }
    }
}
