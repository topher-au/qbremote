use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use qbt_api::{
    Capabilities, Client, ClientConfig, Error,
    models::{
        BuildInfo,
        log::{LogEntry, PeerLogEntry},
        search::SearchResult,
        search::SearchStatus,
        sync::{MainDataDelta, PeersDelta},
        torrents::{TorrentFile, TorrentProperties, Tracker, WebSeed},
    },
};
use tokio::{sync::mpsc, task::JoinHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    /// A poll failed; retrying with backoff.
    Retrying {
        attempt: u32,
        error: String,
    },
    /// Unrecoverable (bad credentials, ban, bad configuration). Not retried.
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub version: String,
    pub webapi_version: String,
    pub capabilities: Capabilities,
    pub build: BuildInfo,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub id: u64,
    pub ok: bool,
    /// Server-supplied error text when `ok` is false.
    pub message: String,
}

/// Data produced by a [`Watch`].
#[derive(Debug, Clone)]
pub enum Data {
    Properties(Box<TorrentProperties>),
    Trackers(Vec<Tracker>),
    WebSeeds(Vec<WebSeed>),
    Files(Vec<TorrentFile>),
    PieceStates(Vec<u8>),
    PieceAvailability(Vec<i64>),
    Peers {
        hash: String,
        delta: PeersDelta,
    },
    MainLog(Vec<LogEntry>),
    PeerLog(Vec<PeerLogEntry>),
    SearchStatus(Vec<SearchStatus>),
    SearchResults {
        id: i64,
        offset: i64,
        results: Vec<SearchResult>,
        status: String,
        total: i64,
    },
    RssItems(serde_json::Value),
    CreatorStatus(serde_json::Value),
}

#[derive(Debug, Clone)]
pub enum Event {
    State(ConnectionState),
    Connected(Box<SessionInfo>),
    /// Raw delta; the consumer owns the `MainDataStore` and applies it.
    MainData(Arc<MainDataDelta>),
    Data(Data),
    CommandFinished(CommandResult),
}

/// Detail views that poll only while visible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Watch {
    General(String),
    Trackers(String),
    WebSeeds(String),
    Content(String),
    Peers(String),
    MainLog,
    PeerLog,
    SearchStatus,
    SearchResults(i64),
    Rss,
    Creator,
}

impl Watch {
    fn interval(&self) -> Duration {
        Duration::from_millis(match self {
            Watch::General(_) => 1500,
            Watch::Trackers(_) => 3000,
            Watch::WebSeeds(_) => 5000,
            Watch::Content(_) => 2000,
            Watch::Peers(_) => 2000,
            Watch::MainLog | Watch::PeerLog => 2000,
            Watch::SearchStatus | Watch::SearchResults(_) => 1000,
            Watch::Rss => 30_000,
            Watch::Creator => 1000,
        })
    }
}

/// Dropping the guard stops the poller.
pub struct WatchGuard(JoinHandle<()>);
impl Drop for WatchGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}

struct Inner {
    client: Arc<Client>,
    tx: mpsc::UnboundedSender<Event>,
    rt: tokio::runtime::Handle,
    next_id: AtomicU64,
    multiplier_bits: AtomicU64,
    active: AtomicBool,
}

/// Cloneable handle to a running session.
#[derive(Clone)]
pub struct SessionHandle {
    inner: Arc<Inner>,
    main: Arc<std::sync::Mutex<Option<JoinHandle<()>>>>,
}

pub struct Session;

impl Session {
    /// Start connecting and polling on `rt`. Events arrive on the returned receiver.
    pub fn spawn(
        rt: tokio::runtime::Handle,
        cfg: ClientConfig,
        polling_multiplier: f64,
    ) -> Result<(SessionHandle, mpsc::UnboundedReceiver<Event>), Error> {
        let client = Arc::new(Client::new(cfg)?);
        let (tx, rx) = mpsc::unbounded_channel();
        let inner = Arc::new(Inner {
            client,
            tx,
            rt: rt.clone(),
            next_id: AtomicU64::new(1),
            multiplier_bits: AtomicU64::new(polling_multiplier.max(0.1).to_bits()),
            active: AtomicBool::new(true),
        });
        let task = rt.spawn(run_main(inner.clone()));
        Ok((
            SessionHandle {
                inner,
                main: Arc::new(std::sync::Mutex::new(Some(task))),
            },
            rx,
        ))
    }
}

impl SessionHandle {
    pub fn client(&self) -> Arc<Client> {
        self.inner.client.clone()
    }

    pub fn set_polling_multiplier(&self, m: f64) {
        self.inner
            .multiplier_bits
            .store(m.max(0.1).to_bits(), Ordering::Relaxed);
    }

    /// `false` while the window is minimized: polling slows down.
    pub fn set_active(&self, active: bool) {
        self.inner.active.store(active, Ordering::Relaxed);
    }

    /// Run a mutation off the UI thread. Returns a correlation id; the result arrives as
    /// [`Event::CommandFinished`].
    pub fn run<F, Fut>(&self, f: F) -> u64
    where
        F: FnOnce(Arc<Client>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), Error>> + Send + 'static,
    {
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let inner = self.inner.clone();
        self.inner.rt.spawn(async move {
            let r = f(inner.client.clone()).await;
            let (ok, message) = match r {
                Ok(()) => (true, String::new()),
                Err(e) => (false, e.to_string()),
            };
            let _ = inner
                .tx
                .send(Event::CommandFinished(CommandResult { id, ok, message }));
        });
        id
    }

    /// Start polling a detail view.
    pub fn watch(&self, w: Watch) -> WatchGuard {
        let inner = self.inner.clone();
        WatchGuard(self.inner.rt.spawn(run_watch(inner, w)))
    }

    /// Stop polling and log out (cookie mode), best effort.
    pub async fn close(&self) {
        if let Some(t) = self.main.lock().unwrap().take() {
            t.abort();
        }
        let _ = self.inner.client.logout().await;
        let _ = self
            .inner
            .tx
            .send(Event::State(ConnectionState::Disconnected));
    }
}

impl Inner {
    fn emit(&self, e: Event) -> bool {
        self.tx.send(e).is_ok()
    }
    fn scale(&self, base: Duration) -> Duration {
        let m = f64::from_bits(self.multiplier_bits.load(Ordering::Relaxed));
        let slow = if self.active.load(Ordering::Relaxed) {
            1.0
        } else {
            4.0
        };
        base.mul_f64(m * slow).max(Duration::from_millis(250))
    }
}

fn backoff(attempt: u32) -> Duration {
    Duration::from_millis((1000u64 << attempt.min(5)).min(30_000))
}

/// Errors that retrying cannot fix.
fn is_fatal(e: &Error) -> bool {
    matches!(
        e,
        Error::Unauthorized | Error::Banned | Error::Config(_) | Error::Forbidden(_)
    )
}

async fn connect(inner: &Inner) -> Result<SessionInfo, Error> {
    inner.client.login().await?;
    let webapi_version = inner.client.app_webapi_version().await?;
    let capabilities = inner.client.capabilities().await?;
    if !capabilities.supported {
        return Err(Error::Config(format!(
            "this server speaks WebAPI {webapi_version}, but qbremote needs qBittorrent 5.0 or newer (WebAPI {} or newer)",
            qbt_api::capabilities::MIN_SUPPORTED
        )));
    }
    let version = inner.client.app_version().await?;
    let build = inner.client.app_build_info().await.unwrap_or_default();
    Ok(SessionInfo {
        version,
        webapi_version,
        capabilities,
        build,
    })
}

async fn run_main(inner: Arc<Inner>) {
    let mut attempt = 0u32;
    'outer: loop {
        inner.emit(Event::State(ConnectionState::Connecting));
        match connect(&inner).await {
            Ok(info) => {
                attempt = 0;
                inner.emit(Event::Connected(Box::new(info)));
                inner.emit(Event::State(ConnectionState::Connected));
            }
            Err(e) if is_fatal(&e) => {
                inner.emit(Event::State(ConnectionState::Failed(e.to_string())));
                return;
            }
            Err(e) => {
                attempt += 1;
                inner.emit(Event::State(ConnectionState::Retrying {
                    attempt,
                    error: e.to_string(),
                }));
                tokio::time::sleep(backoff(attempt)).await;
                continue;
            }
        }
        let mut rid = 0;
        let mut interval = Duration::from_millis(1500);
        loop {
            match inner.client.sync_maindata(rid).await {
                Ok(d) => {
                    if attempt > 0 {
                        attempt = 0;
                        inner.emit(Event::State(ConnectionState::Connected));
                    }
                    rid = d.rid;
                    if let Some(ms) = d
                        .server_state
                        .get("refresh_interval")
                        .and_then(|v| v.as_u64())
                    {
                        interval = Duration::from_millis(ms.clamp(500, 60_000));
                    }
                    if !inner.emit(Event::MainData(Arc::new(d))) {
                        return; // receiver gone
                    }
                    tokio::time::sleep(inner.scale(interval)).await;
                }
                Err(e) if is_fatal(&e) => {
                    inner.emit(Event::State(ConnectionState::Failed(e.to_string())));
                    return;
                }
                Err(e) => {
                    attempt += 1;
                    inner.emit(Event::State(ConnectionState::Retrying {
                        attempt,
                        error: e.to_string(),
                    }));
                    tokio::time::sleep(backoff(attempt)).await;
                    // After an outage reconnect from scratch to get a full update.
                    continue 'outer;
                }
            }
        }
    }
}

async fn run_watch(inner: Arc<Inner>, w: Watch) {
    let c = inner.client.clone();
    let mut peers_rid = 0;
    let mut log_id = -1;
    let mut peer_log_id = -1;
    let mut search_offset = 0;
    loop {
        let r: Result<Vec<Data>, Error> = async {
            Ok(match &w {
                Watch::General(h) => vec![
                    Data::Properties(Box::new(c.torrents_properties(h).await?)),
                    Data::PieceStates(c.torrents_piece_states(h).await?),
                    Data::PieceAvailability(c.torrents_piece_availability(h).await?),
                ],
                Watch::Trackers(h) => vec![Data::Trackers(c.torrents_trackers(h).await?)],
                Watch::WebSeeds(h) => vec![Data::WebSeeds(c.torrents_webseeds(h).await?)],
                Watch::Content(h) => vec![Data::Files(c.torrents_files(h, None).await?)],
                Watch::Peers(h) => {
                    let d = c.sync_torrent_peers(h, peers_rid).await?;
                    peers_rid = d.rid;
                    vec![Data::Peers {
                        hash: h.clone(),
                        delta: d,
                    }]
                }
                Watch::MainLog => {
                    let v = c.log_main(None, log_id).await?;
                    if let Some(id) = v.iter().filter_map(|e| e.id).max() {
                        log_id = id;
                    }
                    vec![Data::MainLog(v)]
                }
                Watch::PeerLog => {
                    let v = c.log_peers(peer_log_id).await?;
                    if let Some(id) = v.iter().filter_map(|e| e.id).max() {
                        peer_log_id = id;
                    }
                    vec![Data::PeerLog(v)]
                }
                Watch::SearchStatus => vec![Data::SearchStatus(c.search_status(None).await?)],
                Watch::SearchResults(id) => {
                    let r = c.search_results(*id, 500, search_offset).await?;
                    let results = r.results.unwrap_or_default();
                    let offset = search_offset;
                    search_offset += results.len() as i64;
                    vec![Data::SearchResults {
                        id: *id,
                        offset,
                        results,
                        status: r.status.unwrap_or_default(),
                        total: r.total.unwrap_or(0),
                    }]
                }
                Watch::Rss => vec![Data::RssItems(c.rss_items(true).await?)],
                Watch::Creator => vec![Data::CreatorStatus(c.creator_status(None).await?)],
            })
        }
        .await;
        match r {
            Ok(items) => {
                for d in items {
                    if !inner.emit(Event::Data(d)) {
                        return;
                    }
                }
            }
            // Detail views fail quietly (e.g. torrent removed); the main loop reports connectivity.
            Err(e) => tracing::debug!("watch {w:?} failed: {e}"),
        }
        tokio::time::sleep(inner.scale(w.interval())).await;
    }
}
