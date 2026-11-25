//! User-facing error text for connection and command failures.

use qbt_api::Error;

/// Message for the connect dialog and toasts. Server-supplied text is preserved where useful.
pub fn friendly(e: &Error) -> String {
    match e {
        Error::Unauthorized => "Authentication failed: wrong username/password or API key. \
            If these are right, the server may be rejecting this Host header: add this host to \
            Web UI → \"Server domains\" on the server."
            .into(),
        Error::Banned => "Your IP address is banned by the server after too many failed logins. \
            Wait for the ban to expire (Web UI → ban duration) or unban it on the server."
            .into(),
        Error::Tls(_) => "The server's TLS certificate is not trusted. Add its CA certificate to \
            the profile (CA file), or use a certificate from a trusted authority."
            .into(),
        Error::Network(e) if e.is_timeout() => "The server did not respond in time.".into(),
        Error::Network(e) if e.is_connect() => "Could not connect to the server. Check the address, port and that the Web UI is enabled.".into(),
        Error::Forbidden(m) => format!("Forbidden by the server{}", if m.is_empty() { String::new() } else { format!(": {m}") }),
        Error::Conflict(m) if m.is_empty() => "The server refused the request (conflict).".into(),
        Error::Conflict(m) => m.clone(),
        Error::BadRequest(m) | Error::UnsupportedMedia(m) | Error::NotFound(m) if !m.is_empty() => m.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_specific() {
        assert!(friendly(&Error::Banned).contains("banned"));
        assert!(friendly(&Error::Unauthorized).contains("Server domains"));
        assert_eq!(
            friendly(&Error::Conflict("Torrent queueing must be enabled".into())),
            "Torrent queueing must be enabled"
        );
        assert!(friendly(&Error::Forbidden(String::new())).starts_with("Forbidden"));
    }
}
