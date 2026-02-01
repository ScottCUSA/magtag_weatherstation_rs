/// Crate-wide result alias using the unified `AppError`.
pub type Result<T> = core::result::Result<T, AppError>;

/// Unified application error type combining display and weather errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AppError {
    #[error("unable to update display")]
    DisplayError,

    #[error("unable to draw graphics to display buffer")]
    GraphicsError,

    #[error("DNS query failed")]
    DnsQueryFailed,

    #[error("network connection failed")]
    ConnectionFailed,

    #[error("HTTP request failed")]
    HttpRequestFailed,

    #[error("socket read error")]
    SocketReadError,

    #[error("API timeout error")]
    RequestTimeout,

    #[error("JSON parse failed")]
    JsonParseFailed,

    #[error("an unknown error occurred")]
    Other,
}

// Convert serde_json_core parse errors into our AppError so callers can `?` them
impl From<serde_json_core::de::Error> for AppError {
    fn from(_: serde_json_core::de::Error) -> Self {
        AppError::JsonParseFailed
    }
}
