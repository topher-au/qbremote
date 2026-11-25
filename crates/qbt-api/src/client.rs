use std::sync::{Arc, RwLock};

use reqwest::{
    RequestBuilder, StatusCode,
    cookie::Jar,
    header::{HeaderMap, HeaderName, HeaderValue},
};

use crate::{
    AuthMode, ClientConfig, Error, Params, Result,
    endpoints::{self, Endpoint},
};

/// A successful (2xx) response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: StatusCode,
    pub body: Vec<u8>,
}

impl Response {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_slice(&self.body).map_err(|e| Error::Decode(e.to_string()))
    }
}

/// Async WebUI API client. Cheap to share behind an `Arc`.
pub struct Client {
    cfg: ClientConfig,
    unsupported: std::sync::Mutex<std::collections::HashSet<(String, String)>>,
    http: RwLock<reqwest::Client>,
    login_lock: tokio::sync::Mutex<()>,
}

impl Client {
    pub fn new(cfg: ClientConfig) -> Result<Self> {
        cfg.validate()?;
        let http = build_http(&cfg)?;
        Ok(Self {
            cfg,
            unsupported: Default::default(),
            http: RwLock::new(http),
            login_lock: tokio::sync::Mutex::new(()),
        })
    }

    pub fn config(&self) -> &ClientConfig {
        &self.cfg
    }

    /// True once the server answered 404 with an empty body for this action, which means the
    /// action does not exist there (used for endpoints older than the documented changelog).
    pub fn is_unsupported(&self, scope: &str, action: &str) -> bool {
        self.unsupported
            .lock()
            .unwrap()
            .contains(&(scope.to_owned(), action.to_owned()))
    }

    fn http(&self) -> reqwest::Client {
        self.http.read().expect("http lock poisoned").clone()
    }

    /// Replace the HTTP client (and with it the cookie jar), dropping any stale session cookie.
    fn reset_session(&self) -> Result<()> {
        *self.http.write().expect("http lock poisoned") = build_http(&self.cfg)?;
        Ok(())
    }

    fn url(&self, ep: &Endpoint) -> Result<url::Url> {
        self.cfg
            .base_url
            .join(&format!("api/v2/{}/{}", ep.scope, ep.action))
            .map_err(|e| Error::Config(e.to_string()))
    }

    fn authorize(&self, rb: RequestBuilder) -> RequestBuilder {
        match &self.cfg.auth {
            AuthMode::Cookie { .. } => rb,
            AuthMode::ApiKey(k) => rb.bearer_auth(k),
            AuthMode::Basic { username, password } => rb.basic_auth(username, Some(password)),
        }
    }

    /// Log in (cookie mode). A no-op in API-key and Basic modes, which authenticate per request.
    pub async fn login(&self) -> Result<()> {
        let AuthMode::Cookie { username, password } = &self.cfg.auth else {
            return Ok(());
        };
        let _guard = self.login_lock.lock().await;
        self.do_login(username, password).await
    }

    async fn do_login(&self, username: &str, password: &str) -> Result<()> {
        let ep = endpoints::lookup("auth", "login").expect("login endpoint");
        let params = Params::new()
            .str("username", username)
            .str("password", password);
        let resp = self
            .http()
            .post(self.url(ep)?)
            .form(params.pairs())
            .send()
            .await
            .map_err(Error::from_transport)?;
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        match status.as_u16() {
            200 if body.trim().eq_ignore_ascii_case("fails.") => Err(Error::Unauthorized),
            200..=299 => Ok(()),
            401 => Err(Error::Unauthorized), // never retry: repeated failures trigger the IP ban
            403 => Err(Error::Banned),
            s => Err(Error::from_status(s, "auth/login", &body)),
        }
    }

    /// Log out; errors are ignored by callers on exit.
    pub async fn logout(&self) -> Result<()> {
        if matches!(self.cfg.auth, AuthMode::Cookie { .. }) {
            self.call("auth", "logout", Params::new()).await?;
        }
        Ok(())
    }

    /// Send `scope/action` with an explicit HTTP method (authenticated) and return the raw status,
    /// without error mapping. Used by contract tests to check that the method table is right
    /// (e.g. GET on a POST-only action must be 405).
    pub async fn probe_status(&self, scope: &str, action: &str, post: bool) -> Result<u16> {
        let ep = endpoints::lookup(scope, action)
            .ok_or_else(|| Error::Config(format!("unknown endpoint {scope}/{action}")))?;
        let url = self.url(ep)?;
        let http = self.http();
        let rb = if post { http.post(url) } else { http.get(url) };
        let resp = self
            .authorize(rb)
            .send()
            .await
            .map_err(Error::from_transport)?;
        Ok(resp.status().as_u16())
    }

    /// Call any inventoried action. POST-only actions are sent as POST forms, others as GET queries.
    pub async fn call(&self, scope: &str, action: &str, params: Params) -> Result<Response> {
        let ep = endpoints::lookup(scope, action)
            .ok_or_else(|| Error::Config(format!("unknown endpoint {scope}/{action}")))?;
        let url = self.url(ep)?;
        self.send(ep, move |http| {
            if ep.post {
                http.post(url.clone()).form(params.pairs())
            } else {
                http.get(url.clone()).query(params.pairs())
            }
        })
        .await
    }

    /// Multipart POST (`torrents/add`, `torrents/parseMetadata`, `rss/importRules`).
    /// `build` is invoked again on a retry, because a multipart body can be sent only once.
    pub async fn call_multipart(
        &self,
        scope: &str,
        action: &str,
        build: impl Fn() -> reqwest::multipart::Form,
    ) -> Result<Response> {
        let ep = endpoints::lookup(scope, action)
            .ok_or_else(|| Error::Config(format!("unknown endpoint {scope}/{action}")))?;
        debug_assert!(ep.post);
        let url = self.url(ep)?;
        self.send(ep, move |http| http.post(url.clone()).multipart(build()))
            .await
    }

    async fn send(
        &self,
        ep: &Endpoint,
        build: impl Fn(&reqwest::Client) -> RequestBuilder,
    ) -> Result<Response> {
        let resp = self.send_raw(ep, build).await?;
        let status = resp.status();
        let body = resp.bytes().await.map_err(Error::from_transport)?.to_vec();
        Ok(Response { status, body })
    }

    /// Send with session recovery and status mapping; returns the unread success response.
    async fn send_raw(
        &self,
        ep: &Endpoint,
        build: impl Fn(&reqwest::Client) -> RequestBuilder,
    ) -> Result<reqwest::Response> {
        let path = format!("{}/{}", ep.scope, ep.action);
        let mut retried = false;
        loop {
            let resp = self
                .authorize(build(&self.http()))
                .send()
                .await
                .map_err(Error::from_transport)?;
            let status = resp.status();

            // A 403 on an API path means the session expired (or, with Basic auth, a stale cookie
            // is present). Recover once; a second failure surfaces to the caller.
            if status == StatusCode::FORBIDDEN && !retried {
                match &self.cfg.auth {
                    AuthMode::Cookie { username, password } => {
                        retried = true;
                        let _g = self.login_lock.lock().await;
                        self.do_login(username, password).await?;
                        continue;
                    }
                    AuthMode::Basic { .. } => {
                        retried = true;
                        self.reset_session()?;
                        continue;
                    }
                    AuthMode::ApiKey(_) => {}
                }
            }
            if status.is_success() {
                return Ok(resp);
            }
            let body = resp.bytes().await.unwrap_or_default();
            if status == StatusCode::NOT_FOUND && body.iter().all(u8::is_ascii_whitespace) {
                // An unknown action has no body; a missing torrent carries a message.
                self.unsupported
                    .lock()
                    .unwrap()
                    .insert((ep.scope.to_owned(), ep.action.to_owned()));
            }
            return Err(Error::from_status(
                status.as_u16(),
                &path,
                &String::from_utf8_lossy(&body),
            ));
        }
    }

    /// Stream a GET action's response body to `dest` (e.g. `torrents/downloadFile`) without
    /// buffering it in memory. Returns the number of bytes written. Partial files are removed on error.
    pub async fn download_to(
        &self,
        scope: &str,
        action: &str,
        params: Params,
        dest: &std::path::Path,
    ) -> Result<u64> {
        use tokio::io::AsyncWriteExt;
        let ep = endpoints::lookup(scope, action)
            .ok_or_else(|| Error::Config(format!("unknown endpoint {scope}/{action}")))?;
        let url = self.url(ep)?;
        let mut resp = self
            .send_raw(ep, move |http| {
                if ep.post {
                    http.post(url.clone()).form(params.pairs())
                } else {
                    http.get(url.clone()).query(params.pairs())
                }
            })
            .await?;
        let io = |e: std::io::Error| Error::Config(format!("writing {}: {e}", dest.display()));
        let mut f = tokio::fs::File::create(dest).await.map_err(io)?;
        let mut total = 0u64;
        let result: Result<()> = async {
            while let Some(chunk) = resp.chunk().await.map_err(Error::from_transport)? {
                f.write_all(&chunk).await.map_err(io)?;
                total += chunk.len() as u64;
            }
            f.flush().await.map_err(io)
        }
        .await;
        if let Err(e) = result {
            drop(f);
            let _ = tokio::fs::remove_file(dest).await;
            return Err(e);
        }
        Ok(total)
    }
}

fn build_http(cfg: &ClientConfig) -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    for (k, v) in &cfg.extra_headers {
        headers.insert(
            HeaderName::from_bytes(k.as_bytes()).map_err(|e| Error::Config(e.to_string()))?,
            HeaderValue::from_str(v).map_err(|e| Error::Config(e.to_string()))?,
        );
    }
    let mut b = reqwest::Client::builder()
        .cookie_provider(Arc::new(Jar::default()))
        .timeout(cfg.timeout)
        .default_headers(headers)
        .gzip(true);
    if let Some(pem) = &cfg.ca_pem {
        let cert = reqwest::Certificate::from_pem(pem).map_err(|e| Error::Config(e.to_string()))?;
        b = b.add_root_certificate(cert);
    }
    b.build().map_err(Error::from_transport)
}
