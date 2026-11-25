//! Name-based dispatch of UI commands onto typed client methods. Qt-free so it is unit-testable.

use qbt_api::{Client, Error, Hashes, Params, Response, Result, models::FilePriority};
use serde_json::Value;

/// Parse the UI's `a|b|c` / `all` hash string.
pub fn parse_hashes(s: &str) -> Hashes {
    if s == "all" {
        Hashes::All
    } else {
        Hashes::Some(
            s.split('|')
                .filter(|x| !x.is_empty())
                .map(str::to_owned)
                .collect(),
        )
    }
}

fn split(s: &str, sep: char) -> Vec<String> {
    s.split(sep)
        .filter(|x| !x.is_empty())
        .map(str::to_owned)
        .collect()
}

fn num<T: std::str::FromStr>(s: &str) -> Result<T> {
    s.trim()
        .parse()
        .map_err(|_| Error::BadRequest(format!("invalid number {s:?}")))
}

fn flag(s: &str) -> bool {
    matches!(s, "true" | "1" | "yes")
}

/// Every name accepted by [`torrent_action`], used by the UI coverage gate.
pub const TORRENT_ACTIONS: &[&str] = &[
    "start",
    "stop",
    "forceStart",
    "recheck",
    "reannounce",
    "delete",
    "deleteFiles",
    "rename",
    "setComment",
    "setCategory",
    "addTags",
    "removeTags",
    "setTags",
    "setLocation",
    "setSavePath",
    "setDownloadPath",
    "setAutoManagement",
    "setUploadLimit",
    "setDownloadLimit",
    "setShareLimits",
    "setSuperSeeding",
    "toggleSequentialDownload",
    "toggleFirstLastPiecePrio",
    "increasePrio",
    "decreasePrio",
    "topPrio",
    "bottomPrio",
    "addTrackers",
    "editTracker",
    "removeTrackers",
    "addWebSeeds",
    "editWebSeed",
    "removeWebSeeds",
    "addPeers",
    "filePrio",
    "renameFile",
    "renameFolder",
    "setSSLParameters",
];

/// Execute a torrent action. `hashes` is `a|b` or `all`; `arg` is action-specific
/// (multiple values separated by U+001F).
pub async fn torrent_action(c: &Client, action: &str, hashes: &str, arg: &str) -> Result<()> {
    let h = parse_hashes(hashes);
    let one = || h_one(&h);
    let a: Vec<&str> = arg.split('\u{1f}').collect();
    let a0 = a.first().copied().unwrap_or("");
    let a1 = a.get(1).copied().unwrap_or("");
    let a2 = a.get(2).copied().unwrap_or("");
    match action {
        "start" => c.torrents_start(&h).await,
        "stop" => c.torrents_stop(&h).await,
        "forceStart" => c.torrents_set_force_start(&h, flag(a0)).await,
        "recheck" => c.torrents_recheck(&h).await,
        "reannounce" => c.torrents_reannounce(&h, None).await,
        "delete" => c.torrents_delete(&h, false).await,
        "deleteFiles" => c.torrents_delete(&h, true).await,
        "rename" => c.torrents_rename(&one()?, a0).await,
        "setComment" => c.torrents_set_comment(&h, a0).await,
        "setCategory" => c.torrents_set_category(&h, a0).await,
        "addTags" => c.torrents_add_tags(&h, &split(a0, ',')).await,
        "removeTags" => c.torrents_remove_tags(&h, &split(a0, ',')).await,
        "setTags" => c.torrents_set_tags(&h, &split(a0, ',')).await,
        "setLocation" => c.torrents_set_location(&h, a0).await,
        "setSavePath" => c.torrents_set_save_path(&h, a0).await,
        "setDownloadPath" => c.torrents_set_download_path(&h, a0).await,
        "setAutoManagement" => c.torrents_set_auto_management(&h, flag(a0)).await,
        "setUploadLimit" => c.torrents_set_upload_limit(&h, num(a0)?).await,
        "setDownloadLimit" => c.torrents_set_download_limit(&h, num(a0)?).await,
        "setShareLimits" => {
            let (mode, act) = (
                Some(a.get(3).copied().unwrap_or("")).filter(|s| !s.is_empty()),
                Some(a.get(4).copied().unwrap_or("")).filter(|s| !s.is_empty()),
            );
            c.torrents_set_share_limits(&h, num(a0)?, num(a1)?, num(a2)?, mode, act)
                .await
        }
        "setSuperSeeding" => c.torrents_set_super_seeding(&h, flag(a0)).await,
        "toggleSequentialDownload" => c.torrents_toggle_sequential_download(&h).await,
        "toggleFirstLastPiecePrio" => c.torrents_toggle_first_last_piece_prio(&h).await,
        "increasePrio" => c.torrents_increase_prio(&h).await,
        "decreasePrio" => c.torrents_decrease_prio(&h).await,
        "topPrio" => c.torrents_top_prio(&h).await,
        "bottomPrio" => c.torrents_bottom_prio(&h).await,
        "addTrackers" => c.torrents_add_trackers(&one()?, &split(a0, '\n')).await,
        "editTracker" => {
            let tier = if a2.is_empty() { None } else { Some(num(a2)?) };
            c.torrents_edit_tracker(&one()?, a0, Some(a1).filter(|s| !s.is_empty()), tier)
                .await
        }
        "removeTrackers" => c.torrents_remove_trackers(&one()?, &split(a0, '\n')).await,
        "addWebSeeds" => c.torrents_add_webseeds(&one()?, &split(a0, '\n')).await,
        "editWebSeed" => c.torrents_edit_webseed(&one()?, a0, a1).await,
        "removeWebSeeds" => c.torrents_remove_webseeds(&one()?, &split(a0, '\n')).await,
        "addPeers" => c.torrents_add_peers(&h, &split(a0, '\n')).await,
        "filePrio" => {
            let ids: Vec<i64> = split(a0, ',')
                .iter()
                .filter_map(|x| x.parse().ok())
                .collect();
            c.torrents_file_prio(&one()?, &ids, FilePriority::from(num::<i64>(a1)?))
                .await
        }
        "renameFile" => c.torrents_rename_file(&one()?, a0, a1).await,
        "renameFolder" => c.torrents_rename_folder(&one()?, a0, a1).await,
        "setSSLParameters" => {
            let p = qbt_api::models::torrents::SslParameters {
                ssl_certificate: Some(a0.into()),
                ssl_private_key: Some(a1.into()),
                ssl_dh_params: Some(a2.into()),
                ..Default::default()
            };
            c.torrents_set_ssl_parameters(&one()?, &p).await
        }
        other => Err(Error::Config(format!("unknown torrent action {other}"))),
    }
}

fn h_one(h: &Hashes) -> Result<String> {
    match h {
        Hashes::Some(v) if v.len() == 1 => Ok(v[0].clone()),
        _ => Err(Error::BadRequest(
            "this action needs exactly one torrent".into(),
        )),
    }
}

/// Call any inventoried endpoint with parameters given as a JSON object. Arrays are joined with `|`.
pub async fn call_json(c: &Client, scope: &str, action: &str, params: &str) -> Result<Response> {
    let mut p = Params::new();
    if !params.trim().is_empty() {
        let v: Value =
            serde_json::from_str(params).map_err(|e| Error::BadRequest(e.to_string()))?;
        let Value::Object(o) = v else {
            return Err(Error::BadRequest("params must be an object".into()));
        };
        for (k, v) in o {
            let s = match v {
                Value::String(s) => s,
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                Value::Array(a) => a
                    .iter()
                    .map(|x| {
                        x.as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| x.to_string())
                    })
                    .collect::<Vec<_>>()
                    .join("|"),
                Value::Null => continue,
                other => other.to_string(),
            };
            p = p.str(&k, s);
        }
    }
    c.call(scope, action, p).await
}
