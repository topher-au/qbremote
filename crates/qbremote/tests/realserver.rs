//! UI scenario against a REAL qBittorrent server. Ignored by default; `cargo xtask contract` runs it
//! after the contract tests (which seed the server) with QBT_CONTRACT_* set.

use std::process::{Command, Stdio};

#[test]
#[ignore]
fn ui_scenario_passes_against_a_real_server() {
    let Ok(url) = std::env::var("QBT_CONTRACT_URL") else {
        return;
    };
    let user = std::env::var("QBT_CONTRACT_USER").unwrap_or_else(|_| "admin".into());
    let password = std::env::var("QBT_CONTRACT_PASSWORD").unwrap_or_default();
    let host = std::env::var("QBT_CONTRACT_HOST_HEADER").unwrap_or_default();
    let dir = std::env::temp_dir().join(format!("qbremote-real-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = dir.join("config.toml");
    let headers = if host.is_empty() {
        String::new()
    } else {
        format!("extra_headers = [[\"Host\", \"{host}\"]]\n")
    };
    std::fs::write(
        &cfg,
        format!("last_used = \"real\"\n\n[[profile]]\nname = \"real\"\nurl = \"{url}\"\nauth = \"cookie\"\nusername = \"{user}\"\n{headers}"),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_qbremote"))
        .env("QT_QPA_PLATFORM", "offscreen")
        .env("QT_QUICK_CONTROLS_STYLE", "Fusion")
        .env("QT_LOGGING_TO_CONSOLE", "1")
        .env("QT_FORCE_STDERR_LOGGING", "1")
        .env("QBREMOTE_CONFIG", &cfg)
        .env("QBREMOTE_MEMORY_SECRETS", "1")
        .env("QBREMOTE_TEST_SECRETS", format!("real={password}"))
        .env("QBREMOTE_SELFTEST", "real")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run qbremote");
    std::fs::remove_dir_all(&dir).ok();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("SELFTEST RESULT: PASS") && out.status.success(),
        "real-server UI scenario failed:\n{text}"
    );
}
