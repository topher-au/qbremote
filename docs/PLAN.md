# Development plan: a native Rust thin client for qBittorrent

## 1. Goal and scope

Build a native desktop application in Rust that controls a **remote** qBittorrent instance only through its WebUI API (`/api/v2`). It should:

- **Implement every API action.** [api-inventory.md](api-inventory.md) lists all 10 scopes and 133 actions, taken from source at WebAPI 2.17.0.
- **Look and work like the qBittorrent desktop app:**
  - a menu bar and toolbar
  - a filter sidebar
  - a sortable transfer list
  - a tabbed properties panel
  - a status bar
  - Search, RSS and Log tabs
  - an Options dialog
- **Never touch torrent data locally.** The only local file I/O is:
  - reading `.torrent` files the user picks for upload
  - saving files the server sends back (exports, created torrents, `downloadFile`, RSS rule exports)
  - the app's own config

**Non-goals:**
- no embedded libtorrent
- no local download engine
- no reuse of the WebUI's HTML/JS

**Licensing and branding:** the look is only *superficially* similar.
- Do not copy qBittorrent's icons, artwork or Qt `.ui` files (GPL).
- Use a permissive icon set, such as Lucide or Phosphor (both MIT/ISC-style).
- Pick a distinct product name, e.g. **"qbremote"**. "qBittorrent" appears only in phrases like "client for qBittorrent".

## 2. Correctness strategy

The public wiki is **out of date**. For example, it lists `pausedUP` and `isPrivate`, while the server sends `stoppedUP` and `private`, and it leaves out about 20 endpoints (see the table at the top of the inventory). To get the API right:

1. **Pinned source of truth.** The canonical spec is qBittorrent's `src/webui/api/*.cpp` at a pinned commit, currently `e6b69eb` (API 2.17.0). Re-pinning is an explicit task that diffs `WebAPI_Changelog.md` and the controllers.
2. **Recorded fixtures.** Record real JSON responses from real servers into `crates/qbt-api/tests/fixtures/<version>/`, and test every model against them.
3. **Live contract tests.** CI runs `qbittorrent-nox` in a container (official `qbittorrentofficial/qbittorrent-nox` image), tagged per supported version, and calls every endpoint the client uses. A test fails if:
   - any key is unknown or missing, compared with the fixtures, or
   - any endpoint returns 404 or 405 where the client expects it to work.
4. **Coverage gate.** A test parses `api-inventory.md` (or a generated `endpoints.toml`) and asserts two things:
   - every action has a typed client method
   - every client method is reachable from at least one UI command
5. **Lenient decoding, strict encoding.**
   - Response structs use `Option<T>` for every field and `#[serde(flatten)] extra: Map<String, Value>`, so older and newer servers never break deserialization. Unknown keys are logged once at debug level.
   - Request parameters are typed enums and builders, so the client cannot send misspelled keys.
6. **Capability gating.** At login, read `app/version` and `app/webapiVersion` and build a `Capabilities` struct. Features the server doesn't have are greyed out with a tooltip ("requires WebAPI ≥ 2.16.0"). Each capability's minimum version comes from `WebAPI_Changelog.md`.
   - Where the changelog has no entry (older than 2.11.6), detect the feature at runtime: a 404 from that action marks it unavailable.

**Supported servers:** qBittorrent **5.0 and newer**, which use the `start`/`stop` endpoints and the `stopped*` states. Support for 4.x (`pause`/`resume`, `paused*`) would be an optional compatibility shim behind the same `Capabilities` switch. **Confirmed boundary (M0):** the rename landed in WebAPI **2.11.0**. Upstream commit `5d1c249606` ("Use Start/Stop instead of Resume/Pause", 2024-03-25) bumps `API_VERSION` from 2.10.4 to 2.11.0; tag `release-4.6.7` is 2.9.3 (`pauseAction`), `release-5.0.0beta1` is 2.10.4 (still `pauseAction`), `release-5.0.0rc1` is 2.11.2 (`stopAction`). The client therefore requires WebAPI ≥ 2.11.0 and refuses older servers with an explanatory message (`qbt_api::capabilities::MIN_SUPPORTED`). Per-endpoint minimum versions come from `WebAPI_Changelog.md` (`MIN_VERSIONS`).

## 3. Technology stack

Pin all versions in `Cargo.lock`. Use the latest stable crate versions when the project starts.

| Concern | Choice | Why |
|---|---|---|
| GUI toolkit | **Qt 6** (minimum **6.8 LTS**; developed against 6.12) with **Qt Quick / Qt Quick Controls** | Same toolkit as qBittorrent, with platform styles: `Windows`, `macOS`, `Fusion`, and `org.kde.desktop` on KDE when installed. |
| Rust↔Qt bridge | **cxx-qt** + `cxx-qt-lib` + `cxx-qt-build` (0.10.x, the current release) | KDAB's maintained bridge. QObjects, properties, signals, invokables and `QAbstractItemModel` subclasses are written in Rust (`#[base = …]`, `#[inherit]`, `#[cxx_override]`). `CxxQtThread::queue` hands off safely from tokio to the Qt thread. |
| UI markup | **QML** (embedded with `QmlModule::qml_files`, compiled ahead of time by qmlcachegen) | Declarative layout only. All logic stays in Rust (see §4.5). |
| Async/HTTP | `tokio` + `reqwest` with the `cookies`, `multipart`, `json`, `rustls-tls` and `gzip` features | Cookie jar for `QBT_SID_*`, multipart for uploads, no OpenSSL dependency. |
| Serde | `serde`, `serde_json`, `serde_repr` | |
| Errors | `thiserror` in libraries, `anyhow` in the app | |
| File dialogs | `QtQuick.Dialogs` `FileDialog` (native on every platform, XDG portal on Linux) | Used only for **local** .torrent files and save targets. |
| Credentials | `keyring` | Passwords and API keys go to the OS secret store, never to config files. |
| Config | `directories` + `toml` | Profiles: server URL, auth mode, TLS options, UI layout. |
| Charts | QML `Canvas`/`Shape` speed graph fed by a Rust ring buffer | Avoids the GPL/commercial-licensed Qt Graphs module. |
| Notifications / tray | `notify-rust` + QML `SystemTrayIcon` from `Qt.labs.platform` | Optional, behind feature flags. |
| i18n | Qt `qsTr` + `.ts`/`.qm` files (later) | Wrap every QML string in `qsTr` from day one. |
| Testing | `wiremock`, `proptest`, `insta`; Qt Quick Test (`qmltestrunner`) for QML | |

**Build prerequisites:**
- Qt 6 dev packages (`qt6-base`, `qt6-declarative`) with `qmake6` on PATH, or `QMAKE` set
- CMake ≥ 3.24 and a C++ compiler
- on Linux, the mold, lld or gold linker, because cxx-qt doesn't link with GNU ld.bfd

This machine has Qt 6.12.0, mold and lld.

**Licensing:** Qt is used under the LGPLv3. Ship Qt as dynamic libraries (the default on Linux and Windows; on macOS, frameworks in the bundle). Use only LGPL modules: Core, Gui, Qml, Quick, QuickControls2, QuickDialogs2, Network if needed, and Svg. Avoid GPL-only add-ons such as Qt Graphs and Qt Charts. CI enforces this list.

**Why QML, not Qt Widgets:** qBittorrent itself uses Qt **Widgets**, but cxx-qt has no Rust bindings for widgets. Its README rates QWidgets support as "limited": Rust QObjects can be used *from* a C++ widgets app, but widget APIs themselves aren't bound. A Widgets UI would therefore have to be written in C++, which breaks the "develop in Rust" requirement. Qt Quick Controls with desktop styles comes very close to the Widgets look:
- `TableView` + `HorizontalHeaderView` + `ItemSelectionModel`
- `TreeView`, `SplitView`, `MenuBar`, `ToolBar`, `TabBar`
- native `FileDialog`/`MessageDialog`

The other options I rejected:
- **ritual / rust-qt widget bindings:** unmaintained and stuck on Qt 5.
- **A C++ Widgets UI with a Rust core:** most of the code would be C++.

The fallback is in §8.

## 4. Architecture

```
qbremote/                      (cargo workspace)
├─ crates/qbt-api      pure async API client: transport, auth, typed models, one method per action
├─ crates/qbt-sync     rid-based incremental state engine (maindata + torrentPeers), pure and UI-agnostic
├─ crates/qbt-core     app services: Session (connection lifecycle), CommandBus, pollers, Capabilities, profiles
├─ crates/qbt-qt       cxx-qt bridge layer: Rust QObjects (controllers) and item models exposed to QML
├─ crates/qbremote     binary: QGuiApplication + QQmlApplicationEngine startup, the QML module (qml/), resources
└─ xtask               fixture recording, contract tests, endpoint-coverage check, release packaging
```

### 4.1 Threading model

- **Threads:** the main thread runs the Qt event loop (`QGuiApplication::exec`). A tokio multi-thread runtime runs on background threads, started before the QML engine loads.
- **UI → runtime:** QML calls `#[qinvokable]` methods on Rust controller QObjects. Each one turns the call into a typed `Command` (action plus correlation id) and sends it to the runtime over a `tokio::sync::mpsc` channel. Invokables never await or block.
- **Runtime → UI:** each poller owns its state. After each merge it computes a **change set**: inserted, removed and changed rows, with the roles that changed. It sends the change set to the Qt thread with `CxxQtThread::queue(|model: Pin<&mut Model>| …)`.
  - The models are marked `impl cxx_qt::Threading for X {}`. `qt_thread()` is called once on the Qt thread, and the handle (which is `Send + Sync`) is passed to the runtime.
  - Results of `queue(…)` are handled with `.ok()`, because it returns `Err(ThreadingQueueError)` once the QObject has been destroyed (for example, during shutdown).
  - On the Qt thread, the closure calls the matching `begin*/end*` and `dataChanged` methods, so QML updates incrementally and never resets the whole model.
- **Command results:** they come back the same way, and are emitted as Qt signals (`commandFinished(id, ok, message)`) that the QML dialogs and toasts subscribe to.
- **Rule:** QObjects are touched **only** on the Qt thread, since they are neither `Send` nor `Sync`. All shared data crosses threads by value inside `queue` closures.

### 4.2 `qbt-api` — transport and auth

`Client` is built from a profile with these fields:
- `base_url`, which may include a reverse-proxy sub-path such as `https://host/qbt/`
- auth mode: Cookie login, API key or Basic
- TLS: system roots, plus an optional custom CA, plus an optional per-profile pinned self-signed certificate (fingerprint shown to the user for trust-on-first-use)
- timeout
- extra headers

**Methods:** actions marked **P** in the inventory are sent as POST with a form body. All others are sent as GET with a query string. A const table generated from the inventory enforces this, so a 405 can't happen.

**Encoding:**
- booleans as `true`/`false`
- hash lists joined with `|`, using a `Hashes::All` variant for `all`
- tags joined with `,`
- URLs and categories joined with `\n`
- `setPreferences` sends a single `json` field holding a diff object (only the changed keys)

**Auth flow:**
- **Cookie mode:** POST `auth/login`.
  - 200: keep whatever cookie the jar received. Never hard-code a cookie name.
  - 401: wrong credentials. Stop and don't retry, to avoid triggering the ban.
  - 403: banned. Show the ban message and the configured duration if known.
- **API-key mode:** send `Authorization: Bearer <key>` on every request. Never call login.
- **Session expiry:** any 403 on an API path during cookie mode means the session expired. Re-login once, then retry the request once. On a second failure, go to the Disconnected state.
- **Headers:** never send `Origin`, `Referer` or `Sec-Fetch-*`. The CSRF check then passes for a native client.
- **Logout:** POST `auth/logout` on profile switch and on app exit.

`Error` enum: `Network`, `Tls`, `Unauthorized`, `Banned`, `Forbidden`, `NotFound`, `Conflict(msg)`, `BadRequest(msg)`, `UnsupportedMedia(msg)`, `MethodNotAllowed`, `Decode`, `Unsupported(capability)`. The body text the server sends is kept as the user-visible message.

**Models:** one module per scope. Enums such as `TorrentState`, `TrackerStatus`, `FilePriority(0/1/6/7)`, `ShareLimitAction` and `ContentLayout` each have an `Unknown(String)` or `Other(i64)` fallback variant.

**Preferences:** `Preferences` is a typed struct with all 243 keys, each `Option`, plus `extra`. Write-only `web_ui_password` lives in a separate `PreferencesPatch` type. A `Debug` impl redacts secrets.

### 4.3 `qbt-sync` — incremental state

**`MainData` store** (the core of the app):
- Request `sync/maindata?rid=N`.
- If `full_update`, replace everything.
- Otherwise:
  - merge each partial torrent object **field by field** into `HashMap<Hash, serde_json::Map>`, then update the typed `Torrent` view
  - apply `*_removed` lists for torrents, categories, tags and trackers
  - merge `categories` per field
  - merge `server_state` per field
  - replace or merge `trackers{url→hashes}` per key
- Store the new `rid`.

**Polling:**
- The interval comes from `server_state.refresh_interval`, falling back to 1500 ms.
- Polling backs off exponentially when errors occur.
- Polling slows down when the window is minimized.

**`Peers` store:** the same merge pattern over `sync/torrentPeers`, keyed by `ip:port`. It polls only while the Peers tab is visible for the selected torrent.

**Derived indexes**, updated on each merge:
- counts per status filter, computed client-side with the same rules as the server's `filter` values, verified against `torrents/info?filter=` in contract tests
- counts per category, tag and tracker host
- totals

**Invariant, checked by proptest:** applying a sequence of deltas and then the final state equals a fresh `rid=0` snapshot.

### 4.4 Other pollers (active only while their view is visible)

| View | Endpoint(s) | Cadence |
|---|---|---|
| General tab | `torrents/properties`, `pieceStates`, `pieceAvailability` | 1–2 s |
| Trackers tab | `torrents/trackers` | 3 s |
| HTTP Sources tab | `torrents/webseeds` | 5 s |
| Content tab | `torrents/files` | 2 s |
| Log tab | `log/main`, `log/peers` with `last_known_id` | 2 s, append-only |
| Search tab | `search/status`, `search/results` with offset paging | 1 s while running |
| RSS tab | `rss/items?withData=true` | 30 s, plus on demand |
| Torrent creator | `torrentcreator/status` | 1 s while tasks are active |

### 4.5 `qbt-qt` — the Qt bridge layer

This crate is the only one that depends on cxx-qt. `qbt-api`, `qbt-sync` and `qbt-core` stay Qt-free and are tested without a display.

**Item models.** Rust structs subclass Qt base models through `#[qobject] #[base = QAbstractTableModel]` (or `QAbstractListModel`, or `QAbstractItemModel` for trees):
- They override `rowCount`, `columnCount`, `data`, `roleNames`, `headerData`, and for trees `index`/`parent`, using `#[cxx_override]`.
- They call the inherited `beginInsertRows`, `endRemoveRows`, `dataChanged` and similar methods through `#[inherit]`. Those calls are `unsafe` and every `begin*` must be paired with its `end*`, so all of them go through one audited helper, `ModelMutator`.

| Model | Base | Backs |
|---|---|---|
| `TransferListModel` | QAbstractTableModel | Transfer list: one row per torrent, one column per field, roles for display, sort and raw values |
| `SidebarModel` | QAbstractItemModel (tree) | Status, Categories (nested on `/`), Tags and Trackers, with counts |
| `TrackersModel` | QAbstractItemModel (tree) | Trackers tab, with endpoints as child rows |
| `PeersModel` | QAbstractTableModel | Peers tab |
| `WebSeedsModel` | QAbstractListModel | HTTP Sources tab |
| `ContentModel` | QAbstractItemModel (tree) | Content tab and the Add-dialog file preview. Has an editable priority role and a check state |
| `LogModel`, `PeerLogModel` | QAbstractTableModel | Execution log (append-only) |
| `SearchJobsModel`, `SearchResultsModel` | QAbstractListModel / QAbstractTableModel | Search tab |
| `RssTreeModel`, `RssArticlesModel`, `RssRulesModel` | tree / table / list | RSS tab and downloader |
| `CreatorTasksModel` | QAbstractTableModel | Torrent creator |
| `RemoteDirModel` | QAbstractItemModel (lazy tree, `canFetchMore`/`fetchMore`) | Remote path picker (`app/getDirectoryContent`) |

**Sorting and filtering happen in Rust.**
- `qbt-sync` keeps a sorted, filtered index of row ids for the current sidebar selection, search text and sort column.
- The model maps rows through that index and emits minimal `rowsMoved`, insert and remove calls when it changes.
- This avoids depending on `QSortFilterProxyModel`, which cxx-qt-lib doesn't bind, and keeps a 50k-row list fast.

**Controllers** (`#[qobject] #[qml_element]` singletons, each with `#[qproperty]` state, `#[qinvokable]` actions and `#[qsignal]` events):
- `SessionController`: profiles, connect/disconnect, connection state, capabilities exposed as boolean properties so QML can write `enabled: caps.webSeeds`
- `TorrentActions`: one invokable per `torrents/*` mutation
- `TransferController`: speed limits, alt mode, session pause
- `PreferencesController`: loads preferences into a QML-bindable `QVariantMap`, tracks the diff, applies it
- `SearchController`, `RssController`, `LogController`, `CreatorController`, `AppController`

**QML layer:** `qml/` is listed in `QmlModule::qml_files`, so it is embedded under `qrc:/qt/qml/<uri>/` and precompiled. It contains layout, bindings, delegates and dialogs only. Rules:
- no business logic in JavaScript
- no `XMLHttpRequest`
- `qmllint` runs in CI with the generated `cxxqt/qml_modules` import path

**Styling:**
- **Default style:**
  - Windows: `Windows`
  - macOS: `macOS`
  - Linux: `org.kde.desktop` if it is installed (it comes with KDE's qqc2-desktop-style), otherwise `Fusion`
- **User override:** a setting that is applied with `QQuickStyle::setStyle` before the engine loads.
- **Icons:** follow the system theme (`QIcon::fromTheme`) where it is available. A bundled MIT/ISC icon set is the fallback.

**Custom painting:** the piece-state, availability and progress bars are `QQuickPaintedItem` subclasses written in Rust through `#[base = QQuickPaintedItem]` and cxx-qt-lib's `QPainter`. If that binding turns out to be missing something, they become QML `Canvas` items fed by a byte array from Rust. The M2.5 spike decides which.

## 5. UI specification (mirrors qBittorrent's desktop layout)

Everything below is built with Qt Quick Controls:
- **Menus:** `MenuBar`. On macOS the global menu comes from `Qt.labs.platform` `MenuBar`, unless the Qt version in use gives `QtQuick.Controls` `MenuBar` a native backend. Check this in the M2.5 spike.
- **Toolbar and panels:** `ToolBar` and `SplitView`.
- **Transfer list:** `TableView` with `HorizontalHeaderView`. Column resize and reorder use whatever TableView supports in the minimum Qt version; anything missing is filled in with a custom header delegate. The M2.5 spike settles this.
- **Trees and dialogs:** `TreeView`, and `Dialog`/`ApplicationWindow` for dialogs.

### 5.1 Main window

- **Menu bar:**
  - **File:** Add torrent file…, Add torrent link…, Connect/Switch server…, Disconnect, Exit
  - **Edit:** Start, Stop, Start all, Stop all, Pause/Resume session, Delete, Select all
  - **View:** Toolbar, Status bar, Filters sidebar, Search/RSS/Log tabs, Speed in title bar, Lock UI
  - **Tools:** Options…, Torrent creator…, Search plugins…, RSS downloader…, Statistics…, Cookies…
  - **Help:** About (shows the server's version, WebAPI version and `buildInfo`)
- **Toolbar:** Add file, Add link, Delete, Start, Stop, Queue up/down/top/bottom, Options, and a search box that filters by name, regex optional.
- **Tabs:** Transfers | Search | RSS | Execution Log. A tab is hidden when the server lacks the feature or the user turns it off.
- **Sidebar sections:**
  - **Status:** All, Downloading, Seeding, Completed, Running, Stopped, Active, Inactive, Stalled, Stalled uploading, Stalled downloading, Checking, Moving, Errored, each with a count.
  - **Categories:** All, Uncategorized, then a tree when names use `/`. Context menu: add, edit, remove, remove unused.
  - **Tags:** All, Untagged, then each tag. Context menu: add, remove, remove unused.
  - **Trackers:** All, Trackerless, Error, Warning, Other error (from the `has_*` flags), then per host.
- **Transfer list:**
  - **Columns** (all user-sortable, reorderable and hideable; layout saved per profile): #, Name, Size, Total Size, Progress (a bar), Status, Seeds, Peers, Down/Up speed, ETA, Ratio, Popularity, Category, Tags, Added On, Completed On, Tracker, Down/Up limit, Downloaded, Uploaded, Session Down/Up, Remaining, Time Active, Seeding Time, Save Path, Download Path, Completed, Ratio Limit, Last Seen Complete, Last Activity, Availability, Reannounce In, Private, Infohash v1/v2, Comment.
  - **Interaction:** multi-select, keyboard navigation, double-click action (configurable: start/stop or open properties), and drag-and-drop of local .torrent files and magnet text into the list.
  - **Context menu:**
    - Start, Stop, Force start, Delete…
    - Set location…, Rename…, Edit comment…, Category ▸, Tags ▸
    - Automatic torrent management, Limit download/upload rate…, Limit share ratio…
    - Super seeding, Sequential download, First/last piece first
    - Force recheck, Force reannounce
    - Queue ▸
    - Copy ▸ (name, hashes, magnet, comment, content path)
    - Export .torrent…
    - SSL parameters… (only when the capability exists)
- **Properties panel** (bottom, collapsible):
  - **General:** progress bar and piece-state bar (custom painter), availability bar, plus every `properties` field.
  - **Trackers:** a list with endpoints as expandable child rows. Supports add, edit (URL and tier), remove, copy and reannounce one tracker.
  - **Peers:** columns follow the peer keys, with country flags as text or emoji. Supports add peers, ban peer (`transfer/banPeers`) and copy address.
  - **HTTP Sources:** add, edit and remove web seeds.
  - **Content:** a file tree with progress, priority (Do not download/Normal/High/Maximum/Mixed), availability and piece range. Supports rename file or folder, and "Download to this computer…" (`downloadFile`, 2.16.0+, streamed to disk).
  - **Speed:** a graph of global up/down speed and limits, buffered client-side from `server_state`.
- **Status bar:**
  - external IP, if `status_bar_external_ip` is set
  - DHT nodes
  - free space on disk
  - connection status icon (connected, firewalled or disconnected)
  - DL and UL speed with session totals; clicking opens the global limit dialog
  - alternative speed limits toggle
  - session paused indicator
  - server connection indicator: green, amber while retrying, red

### 5.2 Dialogs

- **Connect/Profiles:**
  - Fields: URL, auth mode (password, API key, Basic), username, "remember", TLS options and a "Test connection" button.
  - Error messages for each failure: "IP banned", "Host header rejected — add this host to *Server domains*", "certificate untrusted".
- **Add torrent:** opens for local files, links, magnets and drag-and-drop.
  - **Options:** save path (with a remote browse button), incomplete path, category, tags, rename, content layout, stop condition, start torrent, add to top of queue, skip hash check (`seedMode`), sequential, first/last, limits, share limits, auto TMM.
  - **File tree preview:**
    - Local files go through `torrents/parseMetadata`.
    - Magnets go through `torrents/fetchMetadata`, polled until metadata arrives. Closing the dialog calls `cancelFetchMetadata`.
    - Preview priorities are sent as `filePriorities` when the request has a single source.
  - **Result:** shows `success_count`, `pending_count` and `failure_count`. It also handles 202 (pending), 409 and 415.
- **Remote path picker:** a reusable widget built on `app/getDirectoryContent`, with `mode=dirs` for folders. It shows free space from `app/getFreeSpaceAtPath`. All save-path fields use it, because local file dialogs must never be used for server paths.
- **Options:** pages Behavior, Downloads, Connection, Speed, BitTorrent, RSS, WebUI and Advanced, matching the source's sections. Each page binds exactly the keys in the inventory.
  - Changes are shown as a diff. Apply/OK sends only the changed keys.
  - Server-only concepts are labelled "(on server)".
  - Includes Send test email, API key rotate/delete (the new key is shown once, with copy) and a network interface dropdown (`networkInterfaceList` and `networkInterfaceAddressList`).
  - Options that depend on the client host (`locale`, the alternative WebUI, `status_bar_external_ip`, the confirm-dialog flags) still change the **server's** settings, and the page says so.
- **Torrent creator:**
  - The source path is chosen with the remote picker.
  - Options: format (v1/v2/hybrid), piece size, private, start seeding, optimize alignment, padded file size limit, trackers, web seeds, comment, source, ignore dotfiles.
  - A task list with progress. Each task can be downloaded as a .torrent (`torrentFile`) or deleted.
- **RSS:** a feed and folder tree. Supports add feed or folder, rename/move, edit URL, refresh interval, remove, mark read and refresh.
  - The article list has a details pane with the description rendered as sanitized plain text or minimal HTML.
  - Download article (`torrents/add` with the article URL) opens the Add dialog.
- **RSS downloader:** rule list with add, rename, clone and delete. The editor covers every `ruleDef` field. A matching-articles preview uses `rss/matchingArticles`. Rules can be exported and imported.
- **Search:**
  - Pattern, category and plugin selectors. Several jobs run as sub-tabs.
  - Results are sortable and filterable by seeds and size. Each result can be downloaded (`search/downloadTorrent`, or opened in the Add dialog) or its description link opened in the system browser.
  - **Plugins dialog:** enable/disable, install from URL, uninstall and update. Install from a local file isn't possible because `sources` paths are on the server; explain this in the UI.
- **Others:**
  - Statistics: `alltime_*`, `global_ratio`, cache and queue stats.
  - Cookies manager: `app/cookies` and `setCookies`.
  - Global speed limits: normal and alt, using `getSpeedLimits` and `setSpeedLimits`, with a fallback to the per-direction endpoints.
  - Share-limits dialog.
  - Set-location dialog.
  - Category editor with save path, download path and share limits.
  - Delete confirmation, honouring the server's `confirm_torrent_deletion` and `delete_torrent_content_files` defaults.
  - Shutdown server, which needs explicit confirmation.

### 5.3 Client-only settings

Stored locally, never sent to the server:
- profiles and the last-used profile
- UI layout and columns
- theme (light, dark or system)
- confirmation preferences
- polling multiplier
- notifications on completion, detected from `completion_on` changes in sync
- tray behavior
- magnet-link handler registration (OS-level; register only when the user opts in)

## 6. Milestones

Each milestone ends with green CI (fmt, clippy with `-D warnings`, tests, contract tests) and a demo.

| # | Milestone | Contents | Exit criteria |
|---|---|---|---|
| M0 | Groundwork | Workspace, CI matrix (Linux/Win/macOS), xtask fixture recorder, `qbittorrent-nox` containers for 5.0.x, 5.1.x, latest. Confirm the version boundaries for the 4.x→5.0 renames. Generate `endpoints.toml` from the inventory. | Fixtures recorded for every read endpoint on every target version |
| M1 | `qbt-api` core | Transport, auth (all three modes), error mapping, models for `app`, `auth`, `transfer`, `torrents` (reads), `sync`; Capabilities | Contract tests pass. Ban and expiry behavior covered by wiremock |
| M2 | `qbt-sync` | MainData and Peers stores, derived counts, pollers | Proptest invariant holds. Filter counts match `torrents/info?filter=` on a live server seeded with test torrents |
| M2.5 | Qt bridge spike → foundation | Start this in parallel with M1, using synthetic data, so the biggest risk is tested first. `qbt-qt` crate, `ModelMutator`, `TransferListModel` driven by `qbt-sync` change sets through `CxxQtThread::queue`, controller pattern, style selection, CI builds on all three OSes with Qt 6.8 and latest | Settles every "spike" item in §4.5 and §5 (native menu bar, column reorder, painted items). A 10k-row model with 1 Hz partial updates stays at 60 fps and leaks nothing under ASan/Valgrind |
| M3 | Main window | Layout, sidebar, transfer list, status bar, connect dialog, profiles and keyring | Handles 10k synthetic torrents at 60 fps on reference hardware. Every column decodes. Looks right in the Windows, macOS, Fusion and KDE styles |
| M4 | Torrent actions | Every `torrents/*` mutation, context menus, all properties tabs, Add dialog with metadata preview, remote path picker | Coverage gate: every `torrents/*` action reachable from the UI |
| M5 | Options | All preference pages and diff-apply, API key management, test email | Round trip: set each key → read back → equal (contract test, per version) |
| M6 | Search, RSS, Log | All `search/*`, `rss/*`, `log/*` | Coverage gate for these scopes |
| M7 | Creator and rest | `torrentcreator/*`, cookies, statistics, `clientdata/*` (used to sync the client's UI state across machines, opt-in), shutdown | **100% endpoint coverage gate passes** |
| M8 | Polish and release | Keyboard shortcuts matching qBittorrent's (Ctrl+O, Ctrl+L, Del, Shift+Del, Ctrl+S, Ctrl+P, Alt+O…), accessibility pass, i18n scaffolding, packaging (AppImage/Flatpak, MSI, signed .dmg), crash-safe config | Release checklist complete |

## 7. Testing

- **Unit tests:** encoding of every parameter builder, snapshot-tested with `insta` to show exact form bodies. Enum fallbacks. Redaction of secrets.
- **Fixture tests:** every model decodes every recorded fixture for every supported version, with no `extra` keys left over for the pinned version.
- **Contract tests:** run with `cargo xtask contract --server <version>`. They start a container, seed it with tiny public-domain test torrents generated locally through `torrentcreator` and added stopped, and exercise every action, including the error paths: 404 hash, 409 queueing disabled, 415 bad file and 405 wrong method.
- **Sync tests:** proptest delta and merge invariants, plus replay of recorded delta streams.
- **Model tests:** a Rust test harness runs the cxx-qt models under `QCoreApplication`/`QGuiApplication` (offscreen platform, `QT_QPA_PLATFORM=offscreen`). It checks that the begin/end signal sequences are consistent with the data. This is a Rust port of the checks Qt's `QAbstractItemModelTester` does, or that tester itself, called through a small C++ shim.
- **QML tests:** Qt Quick Test (`qmltestrunner`) runs dialogs and workflows against mock controllers. Screenshot comparisons of the main window run in each built-in style. `qmllint` must report no warnings.
- **Manual matrix:** before each release, test against a reverse-proxied HTTPS server, a self-signed certificate, API-key auth, Basic auth, and a server with host-header validation on.

## 8. Risks and decisions to confirm

1. **The API keeps changing.** WebAPI goes up a minor version for nearly every qBittorrent release. Mitigations: lenient decoding, capability gating, and a re-pin task each time qBittorrent releases (diff the changelog and controllers, record fixtures again).
2. **Qt Quick versus Qt Widgets fidelity.** Quick Controls desktop styles get close to qBittorrent's Widgets look, but not pixel-identical. Dense tables and header behavior need the most care.
   - If M2.5 shows the gap is unacceptable, the fallback is a **C++ Qt Widgets UI** that uses the same cxx-qt models and controllers. Those are already standard `QAbstractItemModel`s and QObjects, so they work in a Widgets app without changes. Only the QML layer would be rewritten.
   - That fallback would add a C++ UI layer, so it needs your sign-off.
2a. **cxx-qt maturity.** Its README warns that the "API changes frequently". Pin the cxx-qt crates to an exact version and confine every use of them to `qbt-qt`, so upgrades touch one crate. The bridge adds C++ build steps: qmake discovery, CMake, and the linker requirement on Linux. CI caches Qt and covers all three OSes from M2.5 onward.
3. **4.x support.** It is out of scope unless requested. Adding it means shims for `pause`/`resume`, `paused*` states and older `torrents/add` parameters (`paused`, `root_folder`, `skip_checking`).
4. **Large libraries.** The sync engine has to handle 50k torrents. Benchmark merge cost in M2, and keep the typed views incremental.
5. **Secrets.** `app/preferences` returns `web_ui_api_key` and the proxy, mail and DynDNS passwords. Never log them, never cache them on disk, and mask them in the UI until the user reveals them.
