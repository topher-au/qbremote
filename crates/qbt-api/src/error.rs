use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the client. Server-supplied body text is kept as the user-visible message.
#[derive(Debug, Error)]
pub enum Error {
    #[error("network error: {0}")]
    Network(#[source] reqwest::Error),
    #[error("TLS error: {0}")]
    Tls(#[source] reqwest::Error),
    #[error("invalid credentials or API key")]
    Unauthorized,
    #[error("IP address is banned by the server (too many failed logins)")]
    Banned,
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unsupported media: {0}")]
    UnsupportedMedia(String),
    #[error("method not allowed for {0}")]
    MethodNotAllowed(String),
    #[error("unexpected HTTP status {status}: {message}")]
    Http { status: u16, message: String },
    #[error("could not decode response: {0}")]
    Decode(String),
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("server does not support {0}")]
    Unsupported(&'static str),
}

impl Error {
    pub(crate) fn from_transport(e: reqwest::Error) -> Self {
        // reqwest does not expose TLS failures as a distinct kind; inspect the source chain.
        let mut src: Option<&dyn std::error::Error> = Some(&e);
        while let Some(s) = src {
            let m = s.to_string().to_lowercase();
            if m.contains("certificate") || m.contains("tls") || m.contains("handshake") {
                return Error::Tls(e);
            }
            src = s.source();
        }
        Error::Network(e)
    }

    pub(crate) fn from_status(status: u16, path: &str, body: &str) -> Self {
        let msg = body.trim().to_owned();
        match status {
            400 => Error::BadRequest(msg),
            401 => Error::Unauthorized,
            403 => Error::Forbidden(msg),
            404 => Error::NotFound(msg),
            405 => Error::MethodNotAllowed(path.to_owned()),
            409 => Error::Conflict(msg),
            415 => Error::UnsupportedMedia(msg),
            _ => Error::Http {
                status,
                message: msg,
            },
        }
    }
}
