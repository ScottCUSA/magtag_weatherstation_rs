use core::fmt::Write as _;

use crate::{
    config::{
        OPENMETEO_LATITUDE, OPENMETEO_LONGITUDE, OPENMETEO_TEMP_UNIT, OPENMETEO_TIMEZONE,
        OPENMETEO_WIND_UNIT,
    },
    error::{AppError, Result},
    network::http::{extract_body, http_get_raw, url_encode_component},
    weather::model::OpenMeteoResponse,
};

use alloc::{string::String, vec::Vec};

const DAILY_FIELDS: &str = "weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,wind_speed_10m_max,wind_gusts_10m_max,wind_direction_10m_dominant";
const HEADERS_STR: &str = "Accept: application/json";
pub const OPEN_METEO_URL: &str = "api.open-meteo.com";

/// Fetch weather from Open-Meteo using the provided network `stack`.
///
/// Returns a parsed `OpenMeteoResponse` on success or an error `Result` on failure.
pub async fn fetch_weather(stack: embassy_net::Stack<'static>) -> Result<OpenMeteoResponse> {
    let buf = fetch_weather_data(
        stack,
        OPENMETEO_LATITUDE,
        OPENMETEO_LONGITUDE,
        OPENMETEO_TIMEZONE,
        OPENMETEO_TEMP_UNIT,
        OPENMETEO_WIND_UNIT,
    )
    .await
    .map_err(|e| {
        log::error!("Fetching weather data failed: {:?}", e);
        e
    })?;

    let parsed = OpenMeteoResponse::try_from(extract_body(&buf)).map_err(|e| {
        log::error!("Failed to parse JSON response: {:?}", e);
        AppError::from(e)
    })?;

    log::debug!("{parsed:?}");

    Ok(parsed)
}

/// Fetch weather data for a custom latitude, longitude and timezone.
async fn fetch_weather_data(
    stack: embassy_net::Stack<'static>,
    latitude: &str,
    longitude: &str,
    timezone: &str,
    temperature_unit: &str,
    windspeed_unit: &str,
) -> Result<Vec<u8>> {
    // Build request using custom coordinates/timezone
    let query = build_open_meteo_query(
        latitude,
        longitude,
        timezone,
        temperature_unit,
        windspeed_unit,
    )?;

    // Perform HTTP GET request
    http_get_raw(stack, OPEN_METEO_URL, &query, Some(HEADERS_STR)).await
}

/// Build an Open-Meteo HTTP request for the given latitude, longitude and timezone.
fn build_open_meteo_query(
    latitude: &str,
    longitude: &str,
    timezone: &str,
    temperature_unit: &str,
    windspeed_unit: &str,
) -> Result<String> {
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
