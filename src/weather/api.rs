use core::fmt::Write as _;
use embassy_net::{dns::DnsQueryType, tcp::TcpSocket};
use embassy_time::{Duration, Instant, with_deadline};
use heapless::String;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

use crate::error::AppError;

const RESOLVE_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);
const OPEN_METEO_URL: &str = "api.open-meteo.com";
const DAILY_FIELDS: &str = "weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,wind_speed_10m_max,wind_gusts_10m_max,wind_direction_10m_dominant";
const QUERY_ENCODE_SET: &AsciiSet = &CONTROLS
    // common separators / punctuation / reserved characters:
    .add(b' ')
    .add(b'!')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}')
    .add(b'~');

/// Build an Open-Meteo HTTP request for the given latitude, longitude and timezone.
///
/// This function uses `heapless::String` so it works in `no_std` contexts.
/// The query is percent-encoded according to RFC 3986 for characters outside the
/// unreserved set (ALPHA / DIGIT / "-" / "." / "_" / "~").
///
/// Returns a heapless string containing the full HTTP/1.0 request (headers + body).
fn build_open_meteo_request(
    latitude: &str,
    longitude: &str,
    timezone: &str,
) -> Result<String<512>, AppError> {
    let mut lat_enc: String<16> = String::new();
    write!(
        lat_enc,
        "{}",
        utf8_percent_encode(latitude, QUERY_ENCODE_SET)
    )
    .map_err(|_| AppError::HttpRequestFailed)?;
    let mut long_enc: String<16> = String::new();
    write!(
        long_enc,
        "{}",
        utf8_percent_encode(longitude, QUERY_ENCODE_SET)
    )
    .map_err(|_| AppError::HttpRequestFailed)?;
    let mut tz_enc: String<96> = String::new();
    write!(
        tz_enc,
        "{}",
        utf8_percent_encode(timezone, QUERY_ENCODE_SET)
    )
    .map_err(|_| AppError::HttpRequestFailed)?;

    let mut req: String<512> = String::new();
    write!(
        req,
        "GET /v1/forecast?latitude={}&longitude={}&daily={}&timezone={} HTTP/1.0\r\nHost: {}\r\nAccept: application/json\r\n\r\n",
        lat_enc, long_enc, DAILY_FIELDS, tz_enc, OPEN_METEO_URL
    )
    .map_err(|_| AppError::HttpRequestFailed)?;

    Ok(req)
}

/// Fetch weather data using default coordinates/timezone (keeps compatibility with older callers).
pub async fn fetch_weather_data<const N: usize>(
    stack: embassy_net::Stack<'static>,
) -> Result<[u8; N], AppError> {
    // Default values from the original implementation
    fetch_weather_data_with(stack, "39.868", "-104.9719", "America/Denver").await
}

/// Fetch weather data for a custom latitude, longitude and timezone.
///
/// - `latitude` and `longitude` are passed as f64 and formatted with 6 decimal places.
/// - `timezone` is a UTF-8 string and will be percent-encoded when inserted into the URL.
///
/// Returns a fixed-size buffer containing the raw HTTP response bytes (same behaviour as before).
pub async fn fetch_weather_data_with<const N: usize>(
    stack: embassy_net::Stack<'static>,
    latitude: &str,
    longitude: &str,
    timezone: &str,
) -> Result<[u8; N], AppError> {
    let mut rx_buffer = [0u8; N];
    let mut tx_buffer = [0u8; N];

    let ip_addrs = match with_deadline(Instant::now() + RESOLVE_TIMEOUT, async {
        stack.dns_query(OPEN_METEO_URL, DnsQueryType::A).await
    })
    .await
    {
        Ok(Ok(addrs)) => addrs,
        Ok(Err(e)) => {
            log::error!("DNS query failed: {:?}", e);
            log::error!("Cannot resolve {}", OPEN_METEO_URL);
            return Err(AppError::DnsQueryFailed);
        }
        Err(_) => {
            log::error!("DNS query timed out");
            return Err(AppError::DnsQueryFailed);
        }
    };

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(10)));

    let remote_endpoint = (ip_addrs[0], 80);
    log::info!("Connecting to {}...", remote_endpoint.0);
    match with_deadline(Instant::now() + CONNECT_TIMEOUT, async {
        socket.connect(remote_endpoint).await
    })
    .await
    {
        Ok(Ok(())) => {
            // connected
        }
        Ok(Err(e)) => {
            log::error!("Failed to connect: {:?}", e);
            return Err(AppError::ConnectionFailed);
        }
        Err(_) => {
            log::error!("Connection attempt timed out");
            return Err(AppError::ConnectionFailed);
        }
    }

    log::info!("Connected!");
    let mut buf = [0u8; N];

    use embedded_io_async::Write as _;

    // Build request using custom coordinates/timezone
    let request = build_open_meteo_request(latitude, longitude, timezone)?;

    // Send request with a deadline
    match with_deadline(Instant::now() + REQUEST_TIMEOUT, async {
        socket.write_all(request.as_bytes()).await
    })
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            log::error!("Failed to send HTTP request: {:?}", e);
            return Err(AppError::HttpRequestFailed);
        }
        Err(_) => {
            log::error!("Timed out while sending HTTP request");
            return Err(AppError::HttpRequestFailed);
        }
    }

    // Read response with a deadline for the whole receive operation
    match with_deadline(Instant::now() + RESPONSE_TIMEOUT, async {
        loop {
            match socket.read(&mut buf).await {
                Ok(0) => {
                    log::info!("Received complete HTTP response");
                    break Ok(());
                }
                Ok(n) => {
                    log::info!("Read {} bytes", n);
                }
                Err(e) => {
                    log::error!("Socket read error: {:?}", e);
                    break Err(AppError::SocketReadError);
                }
            };
        }
    })
    .await
    {
        Ok(Ok(())) => {}
        Ok(Err(e)) => return Err(e),
        Err(_) => {
            log::error!("Timed out while reading HTTP response");
            return Err(AppError::SocketReadError);
        }
    }

    Ok(buf)
}
