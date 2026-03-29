// Network timeouts
pub const NETWORK_LINK_TIMEOUT_SECS: u64 = 10;
pub const NETWORK_IP_TIMEOUT_SECS: u64 = 20;

// Wifi credentials
pub const WIFI_SSID: &str = env!("WIFI_SSID");
pub const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

// Open-Meteo API weather arguments
pub const OPENMETEO_LATITUDE: &str = "39.868";
pub const OPENMETEO_LONGITUDE: &str = "-104.9719";
pub const OPENMETEO_TIMEZONE: &str = "America/Denver";
pub const OPENMETEO_TEMP_UNIT: &str = "fahrenheit"; // fahrenheit or celsius
pub const OPENMETEO_WIND_UNIT: &str = "mph"; // mph, kmh

// deep sleep constants
pub const SLEEP_ON_ERROR_SECS: u64 = 60 * 5;
pub const SLEEP_ON_SUCCESS_SECS: u64 = 60 * 60 * 24;

// network constants:

pub const RESOLVE_TIMEOUT: embassy_time::Duration = embassy_time::Duration::from_secs(5);
pub const CONNECT_TIMEOUT: embassy_time::Duration = embassy_time::Duration::from_secs(5);
pub const REQUEST_TIMEOUT: embassy_time::Duration = embassy_time::Duration::from_secs(5);
pub const RESPONSE_TIMEOUT: embassy_time::Duration = embassy_time::Duration::from_secs(10);
