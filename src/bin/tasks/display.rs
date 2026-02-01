use embassy_futures::select::{Either, select};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Input, Output},
    rtc_cntl::Rtc,
    spi::master::Spi,
};
use magtag_weatherstation::{
    display::display_error_text, display::display_weather, sleep::enter_deep_sleep_secs,
};

use crate::{NETWORK_ERROR, SLEEP_ON_ERROR_SECS, SLEEP_ON_SUCCESS_SECS, WEATHER_CHANNEL};

#[embassy_executor::task]
pub async fn display_task(
    spi_device: &'static mut ExclusiveDevice<
        Spi<'static, esp_hal::Blocking>,
        Output<'static>,
        Delay,
    >,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
    rtc: &'static mut Rtc<'static>,
) {
    // Wait for either a network error or weather data concurrently.
    match select(NETWORK_ERROR.receive(), WEATHER_CHANNEL.receive()).await {
        Either::First(err_msg) => {
            display_error_text(&err_msg, spi_device, busy, dc, rst);
            enter_deep_sleep_secs(rtc, SLEEP_ON_ERROR_SECS);
        }
        Either::Second(weather_data) => {
            match display_weather(weather_data, spi_device, busy, dc, rst) {
                Ok(_) => {
                    log::info!("Weather display successful, sleeping...");
                    enter_deep_sleep_secs(rtc, SLEEP_ON_SUCCESS_SECS);
                }
                Err(e) => {
                    log::error!("Displaying weather failed: {:?}", e);
                    enter_deep_sleep_secs(rtc, SLEEP_ON_ERROR_SECS);
                }
            }
        }
    }
}
