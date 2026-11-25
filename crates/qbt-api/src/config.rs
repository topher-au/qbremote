use std::{fmt, time::Duration};

use url::Url;

use crate::{Error, Result};

/// How the client authenticates. Secrets are redacted in `Debug` output.
#[derive(Clone)]
pub enum AuthMode {
    /// POST `auth/login`; the session cookie is whatever the server sets.
    Cookie { username: String, password: String },
    /// `Authorization: Bearer <key>` on every request (WebAPI 2.14.1+). Never logs in.
    ApiKey(String),
    /// `Authorization: Basic …` on every request (WebAPI 2.15.0+).
    Basic { username: String, password: String },
}

impl fmt::Debug for AuthMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cookie { username, .. } => write!(f, "Cookie({username}, <redacted>)"),
            Self::ApiKey(_) => f.write_str("ApiKey(<redacted>)"),
            Self::Basic { username, .. } => write!(f, "Basic({username}, <redacted>)"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// May include a reverse-proxy sub-path, e.g. `https://host/qbt/`.
    pub base_url: Url,
    pub auth: AuthMode,
    pub timeout: Duration,
    /// PEM bytes of an extra trusted CA certificate.
    pub ca_pem: Option<Vec<u8>>,
    /// Extra headers. `Origin`, `Referer` and `Sec-Fetch-*` are rejected (they trip the server's CSRF check).
    pub extra_headers: Vec<(String, String)>,
}

impl ClientConfig {
    pub fn new(base_url: &str, auth: AuthMode) -> Result<Self> {
        let mut url = Url::parse(base_url).map_err(|e| Error::Config(e.to_string()))?;
        if !url.path().ends_with('/') {
            let p = format!("{}/", url.path());
            url.set_path(&p);
        }
        Ok(Self {
            base_url: url,
            auth,
            timeout: Duration::from_secs(30),
            ca_pem: None,
            extra_headers: Vec::new(),
        })
    }

    pub(crate) fn validate(&self) -> Result<()> {
        for (name, _) in &self.extra_headers {
            let n = name.to_ascii_lowercase();
            if n == "origin" || n == "referer" || n.starts_with("sec-fetch-") {
                return Err(Error::Config(format!("header `{name}` must not be sent")));
            }
        }
        Ok(())
    }
}
