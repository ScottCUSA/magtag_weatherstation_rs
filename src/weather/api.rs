use embassy_net::{dns::DnsQueryType, tcp::TcpSocket};
use embassy_time::{Duration, Instant, with_deadline};

use crate::error::AppError;

const OPEN_METEO_URL: &str = "api.open-meteo.com";
const OPEN_METEO_REQUEST: &[u8] = b"GET /v1/forecast?latitude=39.868&longitude=-104.9719\
&daily=weather_code,temperature_2m_max,temperature_2m_min,sunrise,sunset,wind_speed_10m_max,\
wind_gusts_10m_max,wind_direction_10m_dominant&timezone=America%2FDenver \
HTTP/1.0\r\nHost: api.open-meteo.com\r\nAccept: application/json\r\n\r\n";

pub async fn fetch_weather_data<const N: usize>(
    stack: embassy_net::Stack<'static>,
) -> Result<[u8; N], AppError> {
    let mut rx_buffer = [0u8; N];
    let mut tx_buffer = [0u8; N];

    let ip_addrs = match with_deadline(Instant::now() + Duration::from_secs(5), async {
        stack.dns_query(OPEN_METEO_URL, DnsQueryType::A).await
    })
    .await
    {
        Ok(Ok(addrs)) => addrs,
        Ok(Err(e)) => {
            log::error!("DNS query failed: {:?}", e);
            log::error!("Cannot resolve api.open-meteo.com");
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
    match with_deadline(Instant::now() + Duration::from_secs(10), async {
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

    // Send request with a deadline
    match with_deadline(Instant::now() + Duration::from_secs(5), async {
        socket.write_all(OPEN_METEO_REQUEST).await
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
    match with_deadline(Instant::now() + Duration::from_secs(10), async {
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
