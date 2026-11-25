# Implementation status

Checked against [PLAN.md](PLAN.md). "Verified" means exercised by a test in this repository; items marked
**not verified** could not be run in the development environment and need a follow-up.

## Milestones

| # | Milestone | State |
|---|---|---|
| M0 | Groundwork | Workspace, CI workflow, `xtask` (endpoints, coverage, record, contract, i18n) and `docs/endpoints.toml` done. 4.x→5.0 boundary **confirmed from source** (WebAPI 2.11.0, see PLAN §2). Fixtures **recorded from real servers** for WebAPI 2.11.2 (qBittorrent 5.0.5-1), 2.11.4 (5.1.4-2) 2.16.2 (`latest`) and 2.17 (`alpha`) with `cargo xtask contract --server <tag>` (secrets redacted), and an offline test decodes them with no unknown keys. All four servers pass the contract suite **and** the real-server UI scenario (`crates/qbremote/tests/realserver.rs`, run by the same command). The pinned commit itself (`e6b69eb`, `API_VERSION` 2.17.0) was **built from source** (`qbittorrent-nox`, GUI off, system libtorrent 2.1.2) and passes the contract suite in strict mode (no unknown keys, exact POST-only method table, filter counts equal) and the real-server UI scenario; the 2.17 fixtures were recorded from that build. The `alpha` image also passes. |
| M1 | `qbt-api` core | Done and verified with wiremock: all auth modes, 401/403 handling, session expiry retry, ban handling, error mapping, no CSRF headers, sub-path URLs, streaming download, capability table from the changelog, runtime 404 detection, all 133 typed methods, typed 243-key `Preferences` and `PreferencesPatch`. |
| M2 | `qbt-sync` | Done. Proptest invariants (deltas == fresh snapshot; row ops transform old order into new). Filters follow upstream `torrentfilter.cpp` / `torrentimpl.cpp`. Benchmarks in release: 50k torrents → merge 14 ms/tick, counts 46 ms, first sorted view 71 ms. Client-side status counts equal the server's `torrents/info?filter=` counts on all three real servers (seeded with stopped magnets). |
| M2.5 | Qt bridge | Done. cxx-qt 0.10, `ModelMutator`-style begin/end helpers, `CxxQtThread::queue` hand-off, style selection. Qt's `QAbstractItemModelTester` (fatal mode) passes on all 32 models under live updates and sort/filter churn, which already found and fixed one real bug. Spike answers: Canvas painting is used for the bars (not `QQuickPaintedItem`); TableView columns resize, show/hide and reorder (drag a header; `TableView.moveColumn`, order saved per profile and synced when opted in). **Measured** (release build, Qt `offscreen` software renderer, 1 Hz updates touching 40% of the rows, `QBREMOTE_PERF=14` against `tools/mock_qbt.py --torrents N`): 10k torrents → ~62 frames/s, worst frame gap 38 ms, store merge 19 ms + model update 1 ms per tick; 50k torrents → ~56 frames/s, worst gap 129 ms, merge 100 ms + model update 6 ms per tick. Getting there needed three fixes found by this measurement: skip re-sorting unless a changed field affects order, signal `dataChanged` only for the columns whose fields changed (713 ms → 91 ms at 10k), and only for rows near the viewport (→ 1 ms). **Valgrind** (`--leak-check=full`, whole UI self-test incl. model testers, debug build, offscreen): the scenario passes; the only definite leak is 1.6 KB inside Qt's raster paint engine and the reported errors are inside Qt's own startup, none in qbremote code (ASan not run). **Not done:** numbers on real GPU hardware. |
| M3 | Main window | Done and checked visually (offscreen screenshots in Fusion and `org.kde.desktop`). **Not verified:** Windows and macOS styles. |
| M4 | Torrent actions | Done: every `torrents/*` action reachable (coverage gate), all properties tabs, Add dialog with metadata preview, remote path picker. Preview JSON shape verified against upstream `serializeTorrentInfo`. |
| M5 | Options | Done: all 244 keys (243 plus write-only `web_ui_password`) editable, diff-apply, API-key rotate/delete, test email, interface lists. The preference round-trip contract test passes against all three real servers. |
| M6 | Search, RSS, Log | Done. |
| M7 | Creator and the rest | Done: creator, cookies, statistics, `clientdata` layout sync (opt-in), shutdown. **100% endpoint gate passes** (two endpoints explicitly superseded by `sync/maindata`). |
| M8 | Polish and release | Shortcuts, accessibility names, i18n scaffolding (495 strings catalogued, translator loading verified), crash-safe config writes done. The **AppImage builds and runs** (`packaging/appimage/build-appimage.sh`, no linuxdeploy needed; verified it loads its bundled Qt and renders live data). The **Flatpak builds and installs** (`packaging/flatpak/build.sh`, KDE 6.10 runtime, per-user; the app starts and resolves all libraries; only smoke-tested, not exercised against a server inside the sandbox). Windows MSI and macOS dmg are scaffolds, never built. |

## Deviations from the plan

* **Tree models are flat lists with a `depth` column** (sidebar, trackers, content, RSS), not
  `QAbstractItemModel` trees, so nodes cannot be collapsed (the sidebar sections can).
* **One generic `GenericTable` model** backs every secondary table instead of one model class per view.
* **The QML module lives in `qbt-qt`**, not in the binary crate, to keep all cxx-qt use in one crate.
* **Many dialogs build parameters in JavaScript** and call endpoints through `AppController.call(...)`;
  the torrent actions and everything stateful are in Rust.
* Qt Test is linked only with the `test-hooks` feature (default for development, off for releases), to keep
  the release on LGPL modules.

## Found by running against real servers

The real containers found things the mock could not: login answers a plain 401 when the `Host` header does not match the server's own (the "Host header rejected" case; the profile and contract runner now send a matching `Host`); 5.x still sends legacy preference keys (`export_dir`, `export_dir_fin`, `mail_notification_ssl_enabled`, `use_subcategories`); `torrents/properties` also carries `has_metadata`, `popularity`, `progress`, `availability`; several endpoints I had no minimum version for (`setCookies`, `addWebSeeds`, `editWebSeed`, `removeWebSeeds`, `setTags`, `setComment`) do not exist on 2.11.2; the POST-only method list grew over time (e.g. `search/downloadTorrent` accepts GET on 2.11.4); and a `RefCell` double-borrow panic in the UI's sync handler that only occurs when an update needs no recount. All fixed and covered by tests.

## Known gaps and risks

* The CI workflow has never been executed on GitHub (its Linux steps were run locally one by one), and
  Windows/macOS builds and styles have never been tried. the MSI and dmg files are scaffolds.
* The client-side tracker-error/warning counts and "Stopped" for errored-but-stopped torrents are
  approximations of server-side state.
* The AppImage bundles ~270 MB of Qt and its dependencies; it relies on the host for glibc, GL/driver and
  X11/Wayland client libraries.
* System tray notifications need a platform tray; they degrade silently where none exists.
* Single-instance forwarding of magnet links to a running window is not implemented.
* qBittorrent 4.x is unsupported by design (refused with an explanatory message).
* Licence: GPL-3.0-or-later (chosen by the owner).
