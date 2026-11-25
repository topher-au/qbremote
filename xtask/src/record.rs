//! `cargo xtask record`: save real server responses as test fixtures
//! (`crates/qbt-api/tests/fixtures/<webapi version>/<scope>.<action>.json`).
//! Secrets in `app/preferences` are replaced with `REDACTED` before anything is written.

use anyhow::{Context, Result, bail};
use qbt_api::{
    AuthMode, Client, ClientConfig, Params, endpoints::ENDPOINTS, models::prefs::SECRET_KEYS,
};
use serde_json::Value;

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

pub fn run(args: &[String]) -> Result<()> {
    let url = flag(args, "--url").context("--url is required")?;
    let auth = match (flag(args, "--api-key"), flag(args, "--user")) {
        (Some(k), _) => AuthMode::ApiKey(k.to_owned()),
        (None, Some(u)) => AuthMode::Cookie {
            username: u.to_owned(),
            password: flag(args, "--password").unwrap_or_default().to_owned(),
        },
        _ => bail!("give --api-key KEY or --user NAME --password PASS"),
    };
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async {
        let mut cfg = ClientConfig::new(url, auth)?;
        if let Some(h) = flag(args, "--host-header") {
            cfg.extra_headers.push(("Host".into(), h.to_owned()));
        }
        let c = Client::new(cfg)?;
        c.login().await?;
        let version = c.app_webapi_version().await?;
        let dir = flag(args, "--out")
            .map(Into::into)
            .unwrap_or_else(|| crate::repo_root().join("crates/qbt-api/tests/fixtures"))
            .join(&version);
        std::fs::create_dir_all(&dir)?;

        // Discover real identifiers to use as parameters.
        let main = c.sync_maindata(0).await?;
        let hash = main.torrents.keys().next().cloned();
        let iface = c.app_network_interfaces().await.ok().and_then(|v| {
            v.get(0)
                .and_then(|i| i.get("value").and_then(Value::as_str).map(str::to_owned))
        });
        println!(
            "recording WebAPI {version} into {} ({} torrents on the server)",
            dir.display(),
            main.torrents.len()
        );

        let mut recorded = 0;
        for e in ENDPOINTS.iter().filter(|e| !e.post) {
            let p = match (e.scope, e.action) {
                (
                    "torrents",
                    "properties" | "trackers" | "webseeds" | "files" | "pieceHashes"
                    | "pieceStates" | "pieceAvailability" | "SSLParameters",
                ) => match &hash {
                    Some(h) => Params::new().str("hash", h.clone()),
                    None => {
                        println!("  skip {}/{} (no torrents)", e.scope, e.action);
                        continue;
                    }
                },
                ("torrents", "uploadLimit" | "downloadLimit") => Params::new().str("hashes", "all"),
                ("sync", "maindata") => Params::new().int("rid", 0),
                ("sync", "torrentPeers") => match &hash {
                    Some(h) => Params::new().str("hash", h.clone()).int("rid", 0),
                    None => continue,
                },
                ("app", "getDirectoryContent") => {
                    Params::new().str("dirPath", "/").str("mode", "dirs")
                }
                ("app", "getFreeSpaceAtPath") => Params::new().str("path", "/"),
                ("app", "networkInterfaceAddressList") => {
                    Params::new().str("iface", iface.clone().unwrap_or_default())
                }
                ("rss", "items") => Params::new().bool("withData", true),
                ("log", "main" | "peers") => Params::new().int("last_known_id", -1),
                ("clientdata", "load") => Params::new().str("keys", "[]"),
                // Binary downloads and endpoints that need prior state are not recorded.
                ("torrents", "export" | "saveMetadata" | "downloadFile")
                | ("torrentcreator", "torrentFile")
                | ("rss", "exportRules" | "matchingArticles")
                | ("search", "results") => continue,
                _ => Params::new(),
            };
            match c.call(e.scope, e.action, p).await {
                Ok(r) => {
                    let mut body: Value = r.json().unwrap_or_else(|_| Value::String(r.text()));
                    if (e.scope, e.action) == ("app", "preferences")
                        && let Value::Object(o) = &mut body
                    {
                        for k in SECRET_KEYS {
                            if o.contains_key(*k) {
                                o.insert((*k).into(), "REDACTED".into());
                            }
                        }
                    }
                    std::fs::write(
                        dir.join(format!("{}.{}.json", e.scope, e.action)),
                        serde_json::to_string_pretty(&body)? + "\n",
                    )?;
                    recorded += 1;
                }
                Err(err) => println!("  {}/{}: {err}", e.scope, e.action),
            }
        }
        println!("recorded {recorded} responses");
        let _ = c.logout().await;
        Ok(())
    })
}
