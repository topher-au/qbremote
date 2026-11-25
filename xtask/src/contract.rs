//! `cargo xtask contract --server TAG`: start the official `qbittorrentofficial/qbittorrent-nox:TAG`
//! container, run `crates/qbt-api/tests/contract.rs` against it, and remove the container.
//! Needs Docker (or Podman via `DOCKER=podman`). Pulls the image on first use.

use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

fn docker() -> String {
    std::env::var("DOCKER").unwrap_or_else(|_| "docker".into())
}

fn out(cmd: &mut Command) -> Result<String> {
    let o = cmd.output()?;
    Ok(format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    ))
}

pub fn run(args: &[String]) -> Result<()> {
    let tag = args
        .iter()
        .position(|a| a == "--server")
        .and_then(|i| args.get(i + 1))
        .context("--server TAG is required (e.g. 5.0.5, 5.1.4, latest)")?;
    let port: u16 = std::net::TcpListener::bind("127.0.0.1:0")?
        .local_addr()?
        .port();
    let name = format!("qbremote-contract-{}", std::process::id());
    let image = format!("qbittorrentofficial/qbittorrent-nox:{tag}");
    let d = docker();

    println!("starting {image} on 127.0.0.1:{port}");
    let st = Command::new(&d)
        .args([
            "run",
            "-d",
            "--rm",
            "--name",
            &name,
            "-p",
            &format!("127.0.0.1:{port}:8080"),
            "-e",
            "QBT_LEGAL_NOTICE=confirm",
            &image,
        ])
        .stdout(Stdio::null())
        .status()?;
    if !st.success() {
        bail!("could not start the container");
    }
    struct Stop(String, String);
    impl Drop for Stop {
        fn drop(&mut self) {
            let _ = Command::new(&self.0)
                .args(["stop", "-t", "2", &self.1])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
    let _guard = Stop(d.clone(), name.clone());

    // 5.x generates a temporary admin password on first start and prints it to the log.
    let mut password = None;
    for _ in 0..60 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let log = out(Command::new(&d).args(["logs", &name]))?;
        if let Some(l) = log.lines().find(|l| l.contains("temporary password")) {
            password = l.rsplit(':').next().map(|s| s.trim().to_owned());
            break;
        }
    }
    let password =
        password.context("the container did not print a temporary password within 60 s")?;

    let password_for_record = password.clone();
    println!("running contract tests against WebAPI of qBittorrent {tag}");
    let st = Command::new("cargo")
        .args([
            "test",
            "-p",
            "qbt-api",
            "--test",
            "contract",
            "--",
            "--ignored",
            "--nocapture",
        ])
        .env("QBT_CONTRACT_URL", format!("http://127.0.0.1:{port}/"))
        .env("QBT_CONTRACT_USER", "admin")
        .env("QBT_CONTRACT_PASSWORD", &password)
        .env("QBT_CONTRACT_STRICT", "1")
        // The server validates Host against its own address (container port 8080).
        .env("QBT_CONTRACT_HOST_HEADER", "localhost:8080")
        .status()?;
    // The same seeded server also runs the UI scenario (the real app, offscreen).
    let ui = Command::new("cargo")
        .args([
            "test",
            "-p",
            "qbremote",
            "--test",
            "realserver",
            "--",
            "--ignored",
            "--nocapture",
        ])
        .env("QBT_CONTRACT_URL", format!("http://127.0.0.1:{port}/"))
        .env("QBT_CONTRACT_USER", "admin")
        .env("QBT_CONTRACT_PASSWORD", &password)
        .env("QBT_CONTRACT_HOST_HEADER", "localhost:8080")
        .status()?;
    // Record read-endpoint responses of this server as fixtures (M0), whatever the test outcome.
    let url = format!("http://127.0.0.1:{port}/");
    let rec: Vec<String> = [
        "--url",
        &url,
        "--user",
        "admin",
        "--password",
        &password_for_record,
        "--host-header",
        "localhost:8080",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    crate::record::run(&rec)?;
    if !ui.success() {
        bail!("real-server UI scenario failed for {tag}");
    }
    if !st.success() {
        bail!("contract tests failed for {tag}");
    }
    Ok(())
}
