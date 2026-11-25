//! Developer tooling: `cargo xtask <command>`.
//!
//! * `endpoints [--check]`  generate (or verify) `docs/endpoints.toml` from the client's endpoint table
//! * `coverage`             fail unless every endpoint has a typed method and a UI path
//! * `record --url U [--api-key K | --user N --password P]`  record read-endpoint JSON as fixtures
//! * `i18n [--release]`     update the translation catalogue (lupdate) and optionally compile it (lrelease)
//! * `contract --server TAG`  run the live contract tests against `qbittorrentofficial/qbittorrent-nox:TAG`

mod contract;
mod coverage;
mod endpoints;
mod i18n;
mod record;

use std::path::PathBuf;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("endpoints") => endpoints::run(args.iter().any(|a| a == "--check")),
        Some("coverage") => coverage::run(),
        Some("record") => record::run(&args[1..]),
        Some("i18n") => i18n::run(args.iter().any(|a| a == "--release")),
        Some("contract") => contract::run(&args[1..]),
        _ => {
            eprintln!(
                "{}",
                include_str!("main.rs")
                    .lines()
                    .take_while(|l| l.starts_with("//"))
                    .map(|l| l.trim_start_matches("//! ").trim_start_matches("//!"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
            std::process::exit(2);
        }
    }
}
