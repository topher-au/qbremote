# qbremote

A native desktop client for a **remote** qBittorrent instance (5.0 or newer), written in Rust with a
Qt Quick UI. It talks only to the Web UI API (`/api/v2`), never touches torrent data locally, and covers
all 133 API actions. It is an independent project and is not affiliated with the qBittorrent project.

See [docs/PLAN.md](docs/PLAN.md) for the design, [docs/api-inventory.md](docs/api-inventory.md) for the API
reference, and [docs/STATUS.md](docs/STATUS.md) for what is done, what deviates from the plan, and what has
not been verified yet.

## Build

Prerequisites: Rust (stable), Qt 6.8+ development packages (`qmake6` on PATH, or set `QMAKE`), CMake ≥ 3.24,
a C++ compiler, and on Linux the `mold`, `lld` or `gold` linker (cxx-qt does not link with GNU `ld.bfd`;
[.cargo/config.toml](.cargo/config.toml) selects mold).

```sh
cargo run -p qbremote                       # development build (includes the UI test hooks)
cargo build --release -p qbremote --no-default-features    # release build: LGPL Qt modules only
```

Run it, press *File → Connect*, and enter the server URL (a reverse-proxy sub-path such as
`https://host/qbt/` works) and either username/password, an API key, or HTTP Basic credentials. Secrets
go to the system keyring, never to the config file.

## Try it without a server

`tools/mock_qbt.py` is a small stand-in for the Web UI API with live-changing torrents:

```sh
python3 tools/mock_qbt.py --port 18080 --api-key test-key
```

Then connect to `http://127.0.0.1:18080/` with API key `test-key`.

## Test

```sh
cargo test --workspace                      # unit, wiremock, proptest and the headless UI self-test
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask endpoints --check               # docs/endpoints.toml matches the client's endpoint table
cargo xtask coverage                        # every endpoint has a typed method and a UI path
cargo test --release -p qbt-sync --test perf -- --nocapture    # 10k / 50k torrent benchmarks
tools/check-qt-modules.sh target/release/qbremote              # release links only LGPL Qt modules

# Frame rate and update cost with N synthetic torrents (prints `perf:` lines, then quits):
python3 tools/mock_qbt.py --port 18091 --api-key k --torrents 10000 &
QBREMOTE_PERF=14 QT_QPA_PLATFORM=offscreen QBREMOTE_MEMORY_SECRETS=1 QBREMOTE_TEST_SECRETS=p=k \
  QBREMOTE_CONFIG=perf.toml ./target/release/qbremote   # perf.toml: one apikey profile "p" → http://127.0.0.1:18091/
```

The UI self-test (`crates/qbremote/tests/selftest.rs`) runs the real application on Qt's `offscreen`
platform against the mock server, with Qt's `QAbstractItemModelTester` attached to every model in fatal
mode. Live contract tests against the official container image are run with
`cargo xtask contract --server 5.1.4-2` (needs Docker); `cargo xtask record` saves real responses as fixtures.

## Layout

| Path | Contents |
|---|---|
| `crates/qbt-api` | Async API client: transport, auth, typed models, one method per action, capability gating |
| `crates/qbt-sync` | `rid`-based incremental state, filters, counts, sorted view diffs (no Qt) |
| `crates/qbt-core` | Sessions, pollers, command dispatch, profiles, secrets, formatting (no Qt) |
| `crates/qbt-qt` | The only crate using cxx-qt: item models, `AppController`, and the QML module (`qml/`) |
| `crates/qbremote` | The binary |
| `xtask` | Endpoint manifest, coverage gate, fixture recorder, contract runner, i18n |
| `tools/` | Mock server, Qt module licence gate |
| `packaging/`, `i18n/` | Flatpak/AppImage/Windows/macOS scaffolds; translation catalogue |

## Licensing

Qt is used under the LGPLv3 as shared libraries and only LGPL modules are linked in release builds
(`tools/check-qt-modules.sh`). Icons are system-theme icons plus an original app icon; no qBittorrent
artwork or `.ui` files are used. This project's own code is licensed under **GPL-3.0-or-later** (see [LICENSE](LICENSE)).
