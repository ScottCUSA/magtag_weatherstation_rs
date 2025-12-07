use core::fmt::Write as _;
use embassy_net::{dns::DnsQueryType, tcp::TcpSocket};
use embassy_time::{Duration, Instant, with_deadline};
use heapless::String;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

use crate::error::AppError;

extern crate alloc;
use alloc::vec::Vec;

const RESOLVE_TIMEOUT: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);
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

pub fn url_encode_component<const N: usize>(component: &str) -> Result<String<N>, AppError> {
    let mut buf = String::new();
    write!(buf, "{}", utf8_percent_encode(component, QUERY_ENCODE_SET))
        .map_err(|_| AppError::Other)?;
    Ok(buf)
}

pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Connect,
    Options,
    Trace,
    Patch,
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Connect => "CONNECT",
            Method::Options => "OPTIONS",
            Method::Trace => "TRACE",
        }
    }
}

/// Returns a heapless string containing the full HTTP/1.0 request (headers + body).
pub fn build_http_request<const N: usize>(
    method: Method,
    target: &str,
    host: &str,
    headers: Option<&str>,
    body: Option<&str>,
) -> Result<String<N>, AppError> {
    let mut req: String<N> = String::new();
    write!(
        req,
        "{} {} HTTP/1.0\r\nHost: {}\r\n",
        method.as_str(),
        target,
        host,
    )
    .map_err(|_| AppError::HttpRequestFailed)?;

    if let Some(h) = headers {
        write!(req, "{}", h).map_err(|_| AppError::HttpRequestFailed)?;
    }

    if let Some(b) = body {
        write!(req, "\r\n{}", b).map_err(|_| AppError::HttpRequestFailed)?;
    }

    write!(req, "\r\n\r\n").map_err(|_| AppError::HttpRequestFailed)?;

    Ok(req)
}

/// Perform an HTTP GET request to the given host with the provided request string.
///
/// This is a low-level HTTP client function that handles DNS resolution, TCP connection,
/// sending the request, and reading the response into a fixed-size buffer.
///
/// Returns a buffer containing the raw HTTP response (headers + body).
pub async fn http_get(
    stack: embassy_net::Stack<'static>,
    host: &str,
    target: &str,
    headers: Option<&str>,
) -> Result<Vec<u8>, AppError> {
    let mut rx_buffer = [0u8; 1536];
    let mut tx_buffer = [0u8; 512];
    // // Use heap-allocated buffers to avoid large stack frames on the embedded target.
    // let mut rx_buffer: Vec<u8> = vec![0; 1536];
    // let mut tx_buffer: Vec<u8> = vec![0; 512];

    let request: String<512> = build_http_request(Method::Get, target, host, headers, None)?;

    log::debug!("resolving IP for {}...", host);

    let ip_addrs = match with_deadline(Instant::now() + RESOLVE_TIMEOUT, async {
        stack.dns_query(host, DnsQueryType::A).await
    })
    .await
    {
        Ok(Ok(addrs)) => addrs,
        Ok(Err(e)) => {
            log::error!("DNS query failed: {:?}", e);
            log::error!("Cannot resolve {}", host);
            return Err(AppError::DnsQueryFailed);
        }
        Err(_) => {
            log::error!("DNS query timed out");
            return Err(AppError::RequestTimeout);
        }
    };

    log::debug!("resolved IP(s) for {:?}...", ip_addrs);

    let mut socket = TcpSocket::new(stack, &mut rx_buffer[..], &mut tx_buffer[..]);
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
            return Err(AppError::RequestTimeout);
        }
    }

    log::info!("Connected!");

    use embedded_io_async::Write as _;

    log::debug!("Sending HTTP request: {}", request);

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
            return Err(AppError::RequestTimeout);
        }
    }

    log::debug!("HTTP request sent");
    log::debug!("Attempting to read response");

    // Read response with a deadline for the whole receive operation. Accumulate into a Vec.
    let mut resp: Vec<u8> = Vec::with_capacity(1536);

    match with_deadline(Instant::now() + RESPONSE_TIMEOUT, async {
        let mut tmp = [0u8; 512];
        loop {
            match socket.read(&mut tmp).await {
                Ok(0) => {
                    log::info!("Received complete HTTP response");
                    break Ok(());
                }
                Ok(n) => {
                    log::info!("Read {} bytes", n);
                    resp.extend_from_slice(&tmp[..n]);
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
            return Err(AppError::RequestTimeout);
        }
    }

    Ok(resp)
}
