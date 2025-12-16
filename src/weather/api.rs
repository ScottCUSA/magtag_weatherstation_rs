use core::fmt::Write as _;

use crate::{
    error::AppError,
    weather::http::{http_get, url_encode_component},
};

extern crate alloc;
use alloc::{string::String, vec::Vec};

const DAILY_FIELDS: &str = "weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,wind_speed_10m_max,wind_gusts_10m_max,wind_direction_10m_dominant";
const HEADERS_STR: &str = "Accept: application/json";
pub const OPEN_METEO_URL: &str = "api.open-meteo.com";

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
) -> Result<String, AppError> {
    let lat_enc: String = url_encode_component(latitude)?;
    let long_enc: String = url_encode_component(longitude)?;
    let tz_enc: String = url_encode_component(timezone)?;

    let mut query: String = String::new();
    write!(
        query,
        "/v1/forecast?latitude={}&longitude={}&daily={}&timezone={}",
        lat_enc, long_enc, DAILY_FIELDS, tz_enc
    )
    .map_err(|_| AppError::HttpRequestFailed)?;
    Ok(query)
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
) -> Result<Vec<u8>, AppError> {
    // Build request using custom coordinates/timezone
    let query = build_open_meteo_query(latitude, longitude, timezone)?;

    // Perform HTTP GET request
    http_get(stack, OPEN_METEO_URL, &query, Some(HEADERS_STR)).await
}
