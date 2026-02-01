use core::fmt::Write as _;

use crate::{
    config::{
        OPENMETEO_LATITUDE, OPENMETEO_LONGITUDE, OPENMETEO_TIMEZONE, TEMPERATURE_UNIT,
        WIND_SPEED_UNIT,
    },
    error::AppError,
    weather::{
        http::{http_get, url_encode_component},
        model::OpenMeteoResponse,
    },
};

extern crate alloc;
use alloc::{string::String, vec::Vec};

const DAILY_FIELDS: &str = "weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,wind_speed_10m_max,wind_gusts_10m_max,wind_direction_10m_dominant";
const HEADERS_STR: &str = "Accept: application/json";
pub const OPEN_METEO_URL: &str = "api.open-meteo.com";

/// Fetch weather and return a parsed `OpenMeteoResponse`.
pub async fn fetch_weather(
    stack: embassy_net::Stack<'static>,
) -> Result<OpenMeteoResponse, AppError> {
    let buf = fetch_weather_data(
        stack,
        OPENMETEO_LATITUDE,
        OPENMETEO_LONGITUDE,
        OPENMETEO_TIMEZONE,
        TEMPERATURE_UNIT,
        WIND_SPEED_UNIT,
    )
    .await
    .map_err(|e| {
        log::error!("Fetching weather data failed: {:?}", e);
        e
    })?;

    match OpenMeteoResponse::try_from(extract_json_payload(&buf)) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            log::error!("Failed to parse JSON response: {:?}", e);
            Err(AppError::JsonParseFailed)
        }
    }
}

/// Fetch weather data for a custom latitude, longitude and timezone.
///
/// - `latitude` and `longitude` are passed as f64 and formatted with 6 decimal places.
/// - `timezone` is a UTF-8 string and will be percent-encoded when inserted into the URL.
///
/// Returns a fixed-size buffer containing the raw HTTP response bytes (same behaviour as before).
pub async fn fetch_weather_data(
    stack: embassy_net::Stack<'static>,
    latitude: &str,
    longitude: &str,
    timezone: &str,
    temperature_unit: &str,
    windspeed_unit: &str,
) -> Result<Vec<u8>, AppError> {
    // Build request using custom coordinates/timezone
    let query = build_open_meteo_query(
        latitude,
        longitude,
        timezone,
        temperature_unit,
        windspeed_unit,
    )?;

    // Perform HTTP GET request
    http_get(stack, OPEN_METEO_URL, &query, Some(HEADERS_STR)).await
}

/// Build an Open-Meteo HTTP request for the given latitude, longitude and timezone.
///
/// This function uses `heapless::String` so it works in `no_std` contexts.
/// The query is percent-encoded according to RFC 3986 for characters outside the
/// unreserved set (ALPHA / DIGIT / "-" / "." / "_" / "~").
///
/// Returns a heapless string containing the full HTTP/1.0 request (headers + body).
pub fn build_open_meteo_query(
    latitude: &str,
    longitude: &str,
    timezone: &str,
    temperature_unit: &str,
    windspeed_unit: &str,
) -> Result<String, AppError> {
    let lat_enc = url_encode_component(latitude)?;
    let long_enc = url_encode_component(longitude)?;
    let tz_enc = url_encode_component(timezone)?;
    let temp_unit_enc = url_encode_component(temperature_unit)?;
    let windspeed_unit_enc = url_encode_component(windspeed_unit)?;

    let mut query: String = String::new();
    write!(
        query,
        "/v1/forecast?latitude={}&longitude={}&daily={}&timezone={}&temperature_unit={}&wind_speed_unit={}",
        lat_enc, long_enc, DAILY_FIELDS, tz_enc, temp_unit_enc, windspeed_unit_enc
    )
    .map_err(|_| AppError::HttpRequestFailed)?;
    Ok(query)
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
