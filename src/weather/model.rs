use heapless::LinearMap;
use heapless::String;
use heapless::Vec;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_json_core::{self as json_core};

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

// Heapless sizing limits
const MAX_DAYS: usize = 7;

// Heuristic string capacities
const BUF_LEN: usize = 32;
const TZ_ABBR_LEN: usize = 8;

/// Meteo API response struct
#[derive(Deserialize, Debug)]
pub struct OpenMeteoResponse {
    pub latitude: f32,
    pub longitude: f32,
    pub generationtime_ms: f32,
    pub utc_offset_seconds: i32,
    pub timezone: String<BUF_LEN>,
    pub timezone_abbreviation: String<TZ_ABBR_LEN>,
    pub elevation: f32,
    pub daily_units: DailyUnits,
    pub daily: Daily,
}

/// Daily weather data struct
#[derive(Deserialize, Debug)]
pub struct Daily {
    pub time: Vec<String<BUF_LEN>, MAX_DAYS>,
    pub weather_code: Vec<i32, MAX_DAYS>,
    pub temperature_2m_max: Vec<f32, MAX_DAYS>,
    pub temperature_2m_min: Vec<f32, MAX_DAYS>,
    pub sunrise: Vec<String<BUF_LEN>, MAX_DAYS>,
    pub sunset: Vec<String<BUF_LEN>, MAX_DAYS>,
    pub wind_speed_10m_max: Vec<f32, MAX_DAYS>,
    pub wind_direction_10m_dominant: Vec<i32, MAX_DAYS>,
}

/// Daily Units
#[derive(Deserialize, Debug)]
pub struct DailyUnits {
    pub time: String<BUF_LEN>,
    pub weather_code: String<BUF_LEN>,
    pub temperature_2m_max: String<BUF_LEN>,
    pub temperature_2m_min: String<BUF_LEN>,
    pub sunrise: String<BUF_LEN>,
    pub sunset: String<BUF_LEN>,
    pub wind_speed_10m_max: String<BUF_LEN>,
    pub wind_direction_10m_dominant: String<BUF_LEN>,
}

/// Parse the weather JSON response into an ApiResponse struct
/// Allow converting a byte slice into an owned, borrowed `ApiResponse` using the
/// standard library conversion trait. This makes the parser usable in generic
/// contexts where a TryFrom impl is expected.
impl core::convert::TryFrom<&[u8]> for OpenMeteoResponse {
    type Error = json_core::de::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // serde_json_core::from_slice returns (T, consumed)
        let (parsed, _consumed) = json_core::from_slice::<OpenMeteoResponse>(value)?;
        Ok(parsed)
    }
}

/// Provide a From impl so callers can do `String::from(&api_response)`.
impl From<&OpenMeteoResponse> for String<1024> {
    /// Build a small human-readable summary using a heapless string
    fn from(parsed: &OpenMeteoResponse) -> Self {
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
