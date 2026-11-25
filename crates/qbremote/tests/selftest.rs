//! End-to-end UI test: runs the real app (offscreen) against `tools/mock_qbt.py` and checks the
//! scripted scenario in `qml/SelfTest.qml`. Skipped when python3 or the display plugin is missing.

use std::{
    io::{BufRead, BufReader},
    net::TcpListener,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Mock(Child);
impl Drop for Mock {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn ui_scenario_passes_against_mock_server() {
    if Command::new("python3").arg("--version").output().is_err() {
        eprintln!("skipping: python3 not available");
        return;
    }
    let port = free_port();
    let mock = Command::new("python3")
        .arg(repo_root().join("tools/mock_qbt.py"))
        .args(["--port", &port.to_string(), "--api-key", "k"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("start mock server");
    let mut mock = Mock(mock);
    // Wait for the banner so the port is listening.
    let mut line = String::new();
    BufReader::new(mock.0.stdout.as_mut().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert!(
        line.contains("mock qBittorrent"),
        "mock did not start: {line}"
    );

    let dir = std::env::temp_dir().join(format!("qbremote-selftest-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let cfg = dir.join("config.toml");
    std::fs::write(
        &cfg,
        format!("last_used = \"t\"\n\n[[profile]]\nname = \"t\"\nurl = \"http://127.0.0.1:{port}/\"\nauth = \"apikey\"\n"),
    )
    .unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_qbremote"))
        .env("QT_QPA_PLATFORM", "offscreen")
        .env("QT_QUICK_CONTROLS_STYLE", "Fusion")
        .env("QT_LOGGING_TO_CONSOLE", "1")
        .env("QT_FORCE_STDERR_LOGGING", "1")
        .env("QBREMOTE_CONFIG", &cfg)
        .env("QBREMOTE_MEMORY_SECRETS", "1")
        .env("QBREMOTE_TEST_SECRETS", "t=k")
        .env("QBREMOTE_SELFTEST", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start qbremote");

    let start = Instant::now();
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        if start.elapsed() > Duration::from_secs(60) {
            let _ = child.kill();
            panic!("self-test timed out");
        }
        std::thread::sleep(Duration::from_millis(200));
    };
    let out = child.wait_with_output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::remove_dir_all(&dir).ok();
    assert!(
        text.contains("SELFTEST RESULT: PASS") && status.success(),
        "UI self-test failed:\n{text}"
    );
}
