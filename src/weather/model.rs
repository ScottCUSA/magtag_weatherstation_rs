use heapless::String;
use heapless::Vec;
use serde::Deserialize;
use serde_json_core::{self as json_core};

use crate::weather::WEATHER_CODES;

// Heapless sizing limits
const MAX_DAYS: usize = 7;

/// Meteo API response struct
#[derive(Deserialize, Debug)]
pub struct OpenMeteoResponse<'a> {
    pub latitude: f32,
    pub longitude: f32,
    pub generationtime_ms: f32,
    pub utc_offset_seconds: i32,
    pub timezone: &'a str,
    pub timezone_abbreviation: &'a str,
    pub elevation: f32,
    pub daily_units: DailyUnits<'a>,
    pub daily: Daily<'a>,
}

/// Daily weather data struct
#[derive(Deserialize, Debug)]
pub struct Daily<'a> {
    #[serde(borrow)]
    pub time: Vec<&'a str, MAX_DAYS>,
    pub weather_code: Vec<i32, MAX_DAYS>,
    pub temperature_2m_max: Vec<f32, MAX_DAYS>,
    pub temperature_2m_min: Vec<f32, MAX_DAYS>,
    pub sunrise: Vec<&'a str, MAX_DAYS>,
    pub sunset: Vec<&'a str, MAX_DAYS>,
    pub wind_speed_10m_max: Vec<f32, MAX_DAYS>,
    pub wind_direction_10m_dominant: Vec<i32, MAX_DAYS>,
}

/// Daily Units
#[derive(Deserialize, Debug)]
pub struct DailyUnits<'a> {
    #[serde(borrow)]
    pub time: &'a str,
    pub weather_code: &'a str,
    pub temperature_2m_max: &'a str,
    pub temperature_2m_min: &'a str,
    pub sunrise: &'a str,
    pub sunset: &'a str,
    pub wind_speed_10m_max: &'a str,
    pub wind_direction_10m_dominant: &'a str,
}

/// Parse the weather JSON response into an ApiResponse struct
/// Allow converting a byte slice into an owned, borrowed `ApiResponse` using the
/// standard library conversion trait. This makes the parser usable in generic
/// contexts where a TryFrom impl is expected.
impl<'de> core::convert::TryFrom<&'de [u8]> for OpenMeteoResponse<'de> {
    type Error = json_core::de::Error;

    fn try_from(value: &'de [u8]) -> Result<Self, Self::Error> {
        // serde_json_core::from_slice returns (T, consumed)
        let (parsed, _consumed) = json_core::from_slice::<OpenMeteoResponse<'de>>(value)?;
        Ok(parsed)
    }
}

/// Provide a From impl so callers can do `String::from(&api_response)`.
impl<'a> From<&OpenMeteoResponse<'a>> for String<1024> {
    /// Build a small human-readable summary using a heapless string
    fn from(parsed: &OpenMeteoResponse<'a>) -> Self {
        let mut out: String<1024> = String::new();
        use core::fmt::Write as _;
        let _ = write!(
            out,
            "{} ({})\nlat: {:.4} lon: {:.4}\n\n",
            parsed.timezone, parsed.timezone_abbreviation, parsed.latitude, parsed.longitude
        );

        for (i, _) in parsed.daily.time.iter().enumerate() {
            let _ = writeln!(
                out,
                "{}  {:.1}C / {:.1}C {}",
                parsed.daily.time[i],
                parsed.daily.temperature_2m_max[i],
                parsed.daily.temperature_2m_min[i],
                WEATHER_CODES
                    .get(&parsed.daily.weather_code[i])
                    .unwrap_or(&"Unknown"),
            );
        }

        out
    }
}
