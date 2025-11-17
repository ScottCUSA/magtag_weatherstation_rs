use core::fmt::{Display, Write as _};
use heapless::String;

/// Unified application error type combining display and weather errors.
#[derive(Debug, Clone)]
pub enum AppError {
    // Display errors
    DisplayError,

    // Weather/network errors
    DnsQueryFailed,
    ConnectionFailed,
    HttpRequestFailed,
    SocketReadError,
    JsonParseFailed,

    // Fallback for unknown errors
    Other,
}

impl Display for AppError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut msg: String<64> = String::new();
        match self {
            AppError::DisplayError => write!(msg, "unable to update display"),
            AppError::DnsQueryFailed => write!(msg, "DNS query failed"),
            AppError::ConnectionFailed => write!(msg, "network connection failed"),
            AppError::HttpRequestFailed => write!(msg, "HTTP request failed"),
            AppError::SocketReadError => write!(msg, "socket read error"),
            AppError::JsonParseFailed => write!(msg, "JSON parse failed"),
            AppError::Other => write!(msg, "an unknown error occurred"),
        }?;
        write!(f, "{}", msg)
    }
}
