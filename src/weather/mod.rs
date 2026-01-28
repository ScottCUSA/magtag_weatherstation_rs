use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    delay::Delay,
    gpio::{Input, Output},
    spi::master::Spi,
};
use once_cell::sync::Lazy;

use heapless::{LinearMap, String, format};

use self::model::ApiResponse;
use crate::{
    config::OPENMETEO_LATITUDE,
    display::{show_app_error, show_on_display},
};
use crate::{
    config::{OPENMETEO_LONGITUDE, OPENMETEO_TIMEZONE},
    error::AppError,
};

pub mod api;
pub mod http;
pub mod model;

// lazy static map for weather codes to descriptions
static WEATHER_CODES: Lazy<LinearMap<i32, &'static str, 25>> = Lazy::new(|| {
    let mut m = LinearMap::new();
    let _ = m.insert(0, "Clear sky");
    let _ = m.insert(1, "Mainly clear");
    let _ = m.insert(2, "Partly cloudy");
    let _ = m.insert(3, "Overcast");
    let _ = m.insert(45, "Fog");
    let _ = m.insert(48, "Rime Fog");
    let _ = m.insert(51, "Light drizzle");
    let _ = m.insert(53, "Moderate drizzle");
    let _ = m.insert(55, "Dense drizzle");
    let _ = m.insert(56, "Light freezing drizzle");
    let _ = m.insert(57, "Freezing drizzle");
    let _ = m.insert(61, "Light rain");
    let _ = m.insert(63, "Moderate rain");
    let _ = m.insert(65, "Heavy rain");
    let _ = m.insert(66, "Light freezing rain");
    let _ = m.insert(67, "Freezing rain");
    let _ = m.insert(71, "Light snow fall");
    let _ = m.insert(73, "Moderate snow fall");
    let _ = m.insert(75, "Heavy snow fall");
    let _ = m.insert(80, "Light rain showers");
    let _ = m.insert(81, "Moderate rain showers");
    let _ = m.insert(82, "Heavy rain showers");
    let _ = m.insert(95, "Thunderstorm");
    let _ = m.insert(96, "Thunderstorm with light hail");
    let _ = m.insert(99, "Thunderstorm with hail");
    m
});

pub async fn fetch_and_display_weather(
    stack: embassy_net::Stack<'static>,
    spi_device: &mut ExclusiveDevice<Spi<'static, esp_hal::Blocking>, Output<'static>, Delay>,
    busy: Input<'static>,
    dc: Output<'static>,
    rst: Output<'static>,
) -> Result<(), AppError> {
    let buf = match api::fetch_weather_data(
        stack,
        OPENMETEO_LATITUDE,
        OPENMETEO_LONGITUDE,
        OPENMETEO_TIMEZONE,
    )
    .await
    {
        Ok(data) => data,
        Err(e) => {
            log::error!("Fetching weather data failed: {:?}", e);
            let error_msg: heapless::String<128> =
                format!("Fetching weather failed: {e}").unwrap_or_default();
            // Attempt to show the error on the display before sleeping. Ignore display errors.
            show_app_error(&error_msg, spi_device, busy, dc, rst);
            return Err(e);
        }
    };

    match ApiResponse::try_from(extract_json_payload(&buf)) {
        Ok(parsed) => {
            log::info!("Parsed response: timezone {}", parsed.timezone);

            #[cfg(feature = "graphical")]
            {
                // Display graphical background
                use crate::graphics::show_background_image;
                let _ = show_background_image(spi_device, busy, dc, rst);
            }

            #[cfg(not(feature = "graphical"))]
            {
                // create the textual summary
                let out = String::from(&parsed);
                let _ = show_on_display(out.as_str(), spi_device, busy, dc, rst);
            }
            Ok(())
        }
        Err(e) => {
            log::info!("Failed to parse JSON response: {:?}", e);

            let mut out: String<512> = String::new();
            use core::fmt::Write as _;
            let _ = write!(out, "JSON parse error\n{:?}", e);

            let _ = show_on_display(out.as_str(), spi_device, busy, dc, rst);

            Err(AppError::JsonParseFailed)
        }
    }
}

/// Extracts the JSON payload from an HTTP response buffer
fn extract_json_payload(buf: &[u8]) -> &[u8] {
    // Find where JSON starts (after HTTP headers or at first JSON character)
    let start = buf
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
        .or_else(|| buf.iter().position(|&b| b == b'{' || b == b'['))
        .unwrap_or(0);

    // Find where the buffer ends (at null byte or end of buffer)
    let end = buf.iter().position(|&b| b == b'\0').unwrap_or(buf.len());

    &buf[start..end]
}
