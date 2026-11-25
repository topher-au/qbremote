//! Typed methods over [`Client::call`]: one per inventoried action.

use serde_json::{Map, Value};

use crate::{
    Capabilities, Client, Error, Hashes, Params, Response, Result, WebApiVersion,
    models::{
        BuildInfo, FilePriority, Preferences,
        log::{LogEntry, PeerLogEntry},
        search::{SearchPlugin, SearchResults, SearchStart, SearchStatus},
        sync::{MainDataDelta, PeersDelta},
        torrents::{
            AddOptions, AddResult, Categories, InfoQuery, SslParameters, Torrent, TorrentFile,
            TorrentProperties, Tracker, WebSeed,
        },
        transfer::{SpeedLimits, TransferInfo},
    },
};

fn json_str(v: &impl serde::Serialize) -> Result<String> {
    serde_json::to_string(v).map_err(|e| Error::Decode(e.to_string()))
}

fn pipe(v: &[String]) -> String {
    v.join("|")
}

/// Generates `fn name(&self, hashes) -> Result<()>` for POST actions taking only `hashes`.
macro_rules! hashes_action {
    ($($(#[$m:meta])* $fn:ident => $action:literal),* $(,)?) => {
        impl Client { $(
            $(#[$m])*
            pub async fn $fn(&self, h: &Hashes) -> Result<()> {
                self.call("torrents", $action, Params::new().hashes("hashes", h)).await?;
                Ok(())
            }
        )* }
    };
}

hashes_action! {
    torrents_start => "start", torrents_stop => "stop", torrents_recheck => "recheck",
    torrents_toggle_sequential_download => "toggleSequentialDownload",
    torrents_toggle_first_last_piece_prio => "toggleFirstLastPiecePrio",
    torrents_increase_prio => "increasePrio", torrents_decrease_prio => "decreasePrio",
    torrents_top_prio => "topPrio", torrents_bottom_prio => "bottomPrio",
}

impl Client {
    async fn unit(&self, scope: &str, action: &str, p: Params) -> Result<()> {
        self.call(scope, action, p).await?;
        Ok(())
    }
    async fn text(&self, scope: &str, action: &str, p: Params) -> Result<String> {
        Ok(self.call(scope, action, p).await?.text().trim().to_owned())
    }

    // ---------------- auth ----------------
    // `login` / `logout` live in client.rs.

    // ---------------- app ----------------
    pub async fn app_version(&self) -> Result<String> {
        self.text("app", "version", Params::new()).await
    }
    pub async fn app_webapi_version(&self) -> Result<String> {
        self.text("app", "webapiVersion", Params::new()).await
    }
    pub async fn app_build_info(&self) -> Result<BuildInfo> {
        self.call("app", "buildInfo", Params::new()).await?.json()
    }
    pub async fn app_process_info(&self) -> Result<Value> {
        self.call("app", "processInfo", Params::new()).await?.json()
    }
    pub async fn app_shutdown(&self) -> Result<()> {
        self.unit("app", "shutdown", Params::new()).await
    }
    pub async fn app_preferences(&self) -> Result<Preferences> {
        self.call("app", "preferences", Params::new()).await?.json()
    }
    /// Sends only the changed keys as a single `json` field.
    pub async fn app_set_preferences(&self, changed: &Map<String, Value>) -> Result<()> {
        self.unit(
            "app",
            "setPreferences",
            Params::new().str("json", json_str(changed)?),
        )
        .await
    }
    pub async fn app_default_save_path(&self) -> Result<String> {
        self.text("app", "defaultSavePath", Params::new()).await
    }
    pub async fn app_send_test_email(&self) -> Result<()> {
        self.unit("app", "sendTestEmail", Params::new()).await
    }
    /// `mode`: `all`, `dirs` or `files`.
    pub async fn app_directory_content(
        &self,
        dir: &str,
        mode: &str,
        with_metadata: bool,
    ) -> Result<Value> {
        self.call(
            "app",
            "getDirectoryContent",
            Params::new()
                .str("dirPath", dir)
                .str("mode", mode)
                .bool("withMetadata", with_metadata),
        )
        .await?
        .json()
    }
    pub async fn app_free_space_at_path(&self, path: &str) -> Result<i64> {
        self.text("app", "getFreeSpaceAtPath", Params::new().str("path", path))
            .await?
            .parse()
            .map_err(|_| Error::Decode("free space".into()))
    }
    pub async fn app_cookies(&self) -> Result<Value> {
        self.call("app", "cookies", Params::new()).await?.json()
    }
    pub async fn app_set_cookies(&self, cookies: &Value) -> Result<()> {
        self.unit(
            "app",
            "setCookies",
            Params::new().str("cookies", json_str(cookies)?),
        )
        .await
    }
    /// Returns the new key. Show it once; never log it.
    pub async fn app_rotate_api_key(&self) -> Result<String> {
        let r = self.call("app", "rotateAPIKey", Params::new()).await?;
        Ok(match r.json::<Value>() {
            Ok(Value::Object(o)) => o
                .get("apiKey")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            Ok(Value::String(s)) => s,
            _ => r.text().trim().to_owned(),
        })
    }
    pub async fn app_delete_api_key(&self) -> Result<()> {
        self.unit("app", "deleteAPIKey", Params::new()).await
    }
    pub async fn app_network_interfaces(&self) -> Result<Value> {
        self.call("app", "networkInterfaceList", Params::new())
            .await?
            .json()
    }
    pub async fn app_network_interface_addresses(&self, iface: &str) -> Result<Value> {
        self.call(
            "app",
            "networkInterfaceAddressList",
            Params::new().str("iface", iface),
        )
        .await?
        .json()
    }

    /// Read the WebAPI version and derive [`Capabilities`].
    pub async fn capabilities(&self) -> Result<Capabilities> {
        let v = self.app_webapi_version().await?;
        let ver: WebApiVersion = v
            .parse()
            .map_err(|_| Error::Decode(format!("bad version {v:?}")))?;
        Ok(Capabilities::from_version(ver))
    }

    // ---------------- log ----------------
    /// `levels` bitmask of `LOG_*`; `None` for the server default (all).
    pub async fn log_main(&self, levels: Option<i64>, last_known_id: i64) -> Result<Vec<LogEntry>> {
        use crate::models::log::*;
        let mut p = Params::new().int("last_known_id", last_known_id);
        if let Some(l) = levels {
            p = p
                .bool("normal", l & LOG_NORMAL != 0)
                .bool("info", l & LOG_INFO != 0)
                .bool("warning", l & LOG_WARNING != 0)
                .bool("critical", l & LOG_CRITICAL != 0);
        }
        self.call("log", "main", p).await?.json()
    }
    pub async fn log_peers(&self, last_known_id: i64) -> Result<Vec<PeerLogEntry>> {
        self.call(
            "log",
            "peers",
            Params::new().int("last_known_id", last_known_id),
        )
        .await?
        .json()
    }

    // ---------------- sync ----------------
    pub async fn sync_maindata(&self, rid: i64) -> Result<MainDataDelta> {
        self.call("sync", "maindata", Params::new().int("rid", rid))
            .await?
            .json()
    }
    pub async fn sync_torrent_peers(&self, hash: &str, rid: i64) -> Result<PeersDelta> {
        self.call(
            "sync",
            "torrentPeers",
            Params::new().str("hash", hash).int("rid", rid),
        )
        .await?
        .json()
    }

    // ---------------- transfer ----------------
    pub async fn transfer_info(&self) -> Result<TransferInfo> {
        self.call("transfer", "info", Params::new()).await?.json()
    }
    pub async fn transfer_speed_limits_mode(&self) -> Result<bool> {
        Ok(self
            .text("transfer", "speedLimitsMode", Params::new())
            .await?
            == "1")
    }
    pub async fn transfer_set_speed_limits_mode(&self, alt: bool) -> Result<()> {
        self.unit(
            "transfer",
            "setSpeedLimitsMode",
            Params::new().int("mode", alt as i32),
        )
        .await
    }
    pub async fn transfer_toggle_speed_limits_mode(&self) -> Result<()> {
        self.unit("transfer", "toggleSpeedLimitsMode", Params::new())
            .await
    }
    pub async fn transfer_download_limit(&self) -> Result<i64> {
        parse_i64(
            self.text("transfer", "downloadLimit", Params::new())
                .await?,
        )
    }
    pub async fn transfer_upload_limit(&self) -> Result<i64> {
        parse_i64(self.text("transfer", "uploadLimit", Params::new()).await?)
    }
    pub async fn transfer_set_download_limit(&self, limit: i64) -> Result<()> {
        self.unit(
            "transfer",
            "setDownloadLimit",
            Params::new().int("limit", limit),
        )
        .await
    }
    pub async fn transfer_set_upload_limit(&self, limit: i64) -> Result<()> {
        self.unit(
            "transfer",
            "setUploadLimit",
            Params::new().int("limit", limit),
        )
        .await
    }
    pub async fn transfer_speed_limits(&self) -> Result<SpeedLimits> {
        self.call("transfer", "getSpeedLimits", Params::new())
            .await?
            .json()
    }
    pub async fn transfer_set_speed_limits(&self, l: &SpeedLimits) -> Result<()> {
        let mut p = Params::new();
        for (k, v) in [
            ("up_limit", l.up_limit),
            ("dl_limit", l.dl_limit),
            ("alt_up_limit", l.alt_up_limit),
            ("alt_dl_limit", l.alt_dl_limit),
        ] {
            if let Some(v) = v {
                p = p.int(k, v);
            }
        }
        self.unit("transfer", "setSpeedLimits", p).await
    }
    /// `peers` entries are `host:port`.
    pub async fn transfer_ban_peers(&self, peers: &[String]) -> Result<()> {
        self.unit(
            "transfer",
            "banPeers",
            Params::new().str("peers", pipe(peers)),
        )
        .await
    }
    pub async fn transfer_pause_session(&self) -> Result<()> {
        self.unit("transfer", "pauseSession", Params::new()).await
    }
    pub async fn transfer_resume_session(&self) -> Result<()> {
        self.unit("transfer", "resumeSession", Params::new()).await
    }

    // ---------------- torrents: reads ----------------
    pub async fn torrents_count(&self) -> Result<i64> {
        parse_i64(self.text("torrents", "count", Params::new()).await?)
    }
    pub async fn torrents_info(&self, q: &InfoQuery) -> Result<Vec<Torrent>> {
        let mut p = Params::new()
            .opt_str("filter", q.filter.as_deref())
            .opt_str("category", q.category.as_deref())
            .opt_str("tag", q.tag.as_deref())
            .opt_str("sort", q.sort.as_deref());
        if let Some(v) = q.reverse {
            p = p.bool("reverse", v);
        }
        if let Some(v) = q.limit {
            p = p.int("limit", v);
        }
        if let Some(v) = q.offset {
            p = p.int("offset", v);
        }
        if let Some(h) = &q.hashes {
            p = p.pipe("hashes", h);
        }
        if let Some(v) = q.private {
            p = p.bool("private", v);
        }
        if let Some(v) = q.include_files {
            p = p.bool("includeFiles", v);
        }
        if let Some(v) = q.include_trackers {
            p = p.bool("includeTrackers", v);
        }
        self.call("torrents", "info", p).await?.json()
    }
    pub async fn torrents_properties(&self, hash: &str) -> Result<TorrentProperties> {
        self.call("torrents", "properties", Params::new().str("hash", hash))
            .await?
            .json()
    }
    pub async fn torrents_trackers(&self, hash: &str) -> Result<Vec<Tracker>> {
        self.call("torrents", "trackers", Params::new().str("hash", hash))
            .await?
            .json()
    }
    pub async fn torrents_webseeds(&self, hash: &str) -> Result<Vec<WebSeed>> {
        self.call("torrents", "webseeds", Params::new().str("hash", hash))
            .await?
            .json()
    }
    pub async fn torrents_files(
        &self,
        hash: &str,
        indexes: Option<&[i64]>,
    ) -> Result<Vec<TorrentFile>> {
        let mut p = Params::new().str("hash", hash);
        if let Some(i) = indexes {
            p = p.str(
                "indexes",
                i.iter().map(i64::to_string).collect::<Vec<_>>().join("|"),
            );
        }
        self.call("torrents", "files", p).await?.json()
    }
    pub async fn torrents_piece_hashes(&self, hash: &str) -> Result<Vec<String>> {
        self.call("torrents", "pieceHashes", Params::new().str("hash", hash))
            .await?
            .json()
    }
    /// 0 missing, 1 downloading, 2 have.
    pub async fn torrents_piece_states(&self, hash: &str) -> Result<Vec<u8>> {
        self.call("torrents", "pieceStates", Params::new().str("hash", hash))
            .await?
            .json()
    }
    pub async fn torrents_piece_availability(&self, hash: &str) -> Result<Vec<i64>> {
        self.call(
            "torrents",
            "pieceAvailability",
            Params::new().str("hash", hash),
        )
        .await?
        .json()
    }
    pub async fn torrents_categories(&self) -> Result<Categories> {
        self.call("torrents", "categories", Params::new())
            .await?
            .json()
    }
    pub async fn torrents_tags(&self) -> Result<Vec<String>> {
        self.call("torrents", "tags", Params::new()).await?.json()
    }
    pub async fn torrents_upload_limits(
        &self,
        h: &Hashes,
    ) -> Result<std::collections::BTreeMap<String, i64>> {
        self.call("torrents", "uploadLimit", Params::new().hashes("hashes", h))
            .await?
            .json()
    }
    pub async fn torrents_download_limits(
        &self,
        h: &Hashes,
    ) -> Result<std::collections::BTreeMap<String, i64>> {
        self.call(
            "torrents",
            "downloadLimit",
            Params::new().hashes("hashes", h),
        )
        .await?
        .json()
    }
    /// The .torrent file bytes.
    pub async fn torrents_export(&self, hash: &str) -> Result<Vec<u8>> {
        Ok(self
            .call("torrents", "export", Params::new().str("hash", hash))
            .await?
            .body)
    }
    pub async fn torrents_ssl_parameters(&self, hash: &str) -> Result<SslParameters> {
        self.call("torrents", "SSLParameters", Params::new().str("hash", hash))
            .await?
            .json()
    }
    pub async fn torrents_save_metadata(&self, source: &str) -> Result<Vec<u8>> {
        Ok(self
            .call(
                "torrents",
                "saveMetadata",
                Params::new().str("source", source),
            )
            .await?
            .body)
    }
    /// Raw response so the caller can stream/save the file bytes (WebAPI 2.16.0+).
    pub async fn torrents_download_file(&self, hash: &str, file: &str) -> Result<Response> {
        self.call(
            "torrents",
            "downloadFile",
            Params::new().str("hash", hash).str("file", file),
        )
        .await
    }

    // ---------------- torrents: mutations ----------------
    pub async fn torrents_reannounce(&self, h: &Hashes, urls: Option<&[String]>) -> Result<()> {
        let mut p = Params::new().hashes("hashes", h);
        if let Some(u) = urls {
            p = p.pipe("urls", u);
        }
        self.unit("torrents", "reannounce", p).await
    }
    pub async fn torrents_delete(&self, h: &Hashes, delete_files: bool) -> Result<()> {
        self.unit(
            "torrents",
            "delete",
            Params::new()
                .hashes("hashes", h)
                .bool("deleteFiles", delete_files),
        )
        .await
    }
    pub async fn torrents_rename(&self, hash: &str, name: &str) -> Result<()> {
        self.unit(
            "torrents",
            "rename",
            Params::new().str("hash", hash).str("name", name),
        )
        .await
    }
    pub async fn torrents_set_comment(&self, h: &Hashes, comment: &str) -> Result<()> {
        self.unit(
            "torrents",
            "setComment",
            Params::new().hashes("hashes", h).str("comment", comment),
        )
        .await
    }
    pub async fn torrents_set_category(&self, h: &Hashes, category: &str) -> Result<()> {
        self.unit(
            "torrents",
            "setCategory",
            Params::new().hashes("hashes", h).str("category", category),
        )
        .await
    }
    /// `extra` carries optional `downloadPathEnabled`, `downloadPath`, share-limit keys, etc.
    pub async fn torrents_create_category(
        &self,
        name: &str,
        save_path: &str,
        extra: &[(&str, String)],
    ) -> Result<()> {
        self.category_write("createCategory", name, save_path, extra)
            .await
    }
    pub async fn torrents_edit_category(
        &self,
        name: &str,
        save_path: &str,
        extra: &[(&str, String)],
    ) -> Result<()> {
        self.category_write("editCategory", name, save_path, extra)
            .await
    }
    async fn category_write(
        &self,
        action: &str,
        name: &str,
        save_path: &str,
        extra: &[(&str, String)],
    ) -> Result<()> {
        let mut p = Params::new()
            .str("category", name)
            .str("savePath", save_path);
        for (k, v) in extra {
            p = p.str(k, v.clone());
        }
        self.unit("torrents", action, p).await
    }
    pub async fn torrents_remove_categories(&self, names: &[String]) -> Result<()> {
        self.unit(
            "torrents",
            "removeCategories",
            Params::new().lines("categories", names),
        )
        .await
    }
    pub async fn torrents_add_tags(&self, h: &Hashes, tags: &[String]) -> Result<()> {
        self.unit(
            "torrents",
            "addTags",
            Params::new().hashes("hashes", h).tags("tags", tags),
        )
        .await
    }
    pub async fn torrents_set_tags(&self, h: &Hashes, tags: &[String]) -> Result<()> {
        self.unit(
            "torrents",
            "setTags",
            Params::new().hashes("hashes", h).tags("tags", tags),
        )
        .await
    }
    pub async fn torrents_remove_tags(&self, h: &Hashes, tags: &[String]) -> Result<()> {
        self.unit(
            "torrents",
            "removeTags",
            Params::new().hashes("hashes", h).tags("tags", tags),
        )
        .await
    }
    pub async fn torrents_create_tags(&self, tags: &[String]) -> Result<()> {
        self.unit("torrents", "createTags", Params::new().tags("tags", tags))
            .await
    }
    pub async fn torrents_delete_tags(&self, tags: &[String]) -> Result<()> {
        self.unit("torrents", "deleteTags", Params::new().tags("tags", tags))
            .await
    }

    /// Add by URL/magnet and/or local .torrent contents. Returns the HTTP status too:
    /// 202 means metadata is still pending.
    pub async fn torrents_add(&self, opts: &AddOptions) -> Result<(u16, AddResult)> {
        let r = self
            .call_multipart("torrents", "add", || {
                let mut f = reqwest::multipart::Form::new();
                if !opts.urls.is_empty() {
                    f = f.text("urls", opts.urls.join("\n"));
                }
                for (name, bytes) in &opts.files {
                    let part = reqwest::multipart::Part::bytes(bytes.clone())
                        .file_name(name.clone())
                        .mime_str("application/x-bittorrent")
                        .expect("static mime");
                    f = f.part("torrents", part);
                }
                for (k, v) in &opts.fields {
                    f = f.text(k.clone(), v.clone());
                }
                f
            })
            .await?;
        let parsed = r
            .json::<AddResult>()
            .unwrap_or_else(|_| AddResult::default());
        Ok((r.status.as_u16(), parsed))
    }
    pub async fn torrents_add_trackers(&self, hash: &str, urls: &[String]) -> Result<()> {
        self.unit(
            "torrents",
            "addTrackers",
            Params::new().str("hash", hash).lines("urls", urls),
        )
        .await
    }
    pub async fn torrents_edit_tracker(
        &self,
        hash: &str,
        url: &str,
        new_url: Option<&str>,
        tier: Option<i64>,
    ) -> Result<()> {
        let mut p = Params::new()
            .str("hash", hash)
            .str("url", url)
            .opt_str("newUrl", new_url);
        if let Some(t) = tier {
            p = p.int("tier", t);
        }
        self.unit("torrents", "editTracker", p).await
    }
    pub async fn torrents_remove_trackers(&self, hash: &str, urls: &[String]) -> Result<()> {
        self.unit(
            "torrents",
            "removeTrackers",
            Params::new().str("hash", hash).pipe("urls", urls),
        )
        .await
    }
    pub async fn torrents_add_webseeds(&self, hash: &str, urls: &[String]) -> Result<()> {
        self.unit(
            "torrents",
            "addWebSeeds",
            Params::new().str("hash", hash).pipe("urls", urls),
        )
        .await
    }
    pub async fn torrents_edit_webseed(&self, hash: &str, orig: &str, new: &str) -> Result<()> {
        self.unit(
            "torrents",
            "editWebSeed",
            Params::new()
                .str("hash", hash)
                .str("origUrl", orig)
                .str("newUrl", new),
        )
        .await
    }
    pub async fn torrents_remove_webseeds(&self, hash: &str, urls: &[String]) -> Result<()> {
        self.unit(
            "torrents",
            "removeWebSeeds",
            Params::new().str("hash", hash).pipe("urls", urls),
        )
        .await
    }
    pub async fn torrents_add_peers(&self, h: &Hashes, peers: &[String]) -> Result<()> {
        self.unit(
            "torrents",
            "addPeers",
            Params::new().hashes("hashes", h).pipe("peers", peers),
        )
        .await
    }
    pub async fn torrents_file_prio(
        &self,
        hash: &str,
        ids: &[i64],
        prio: FilePriority,
    ) -> Result<()> {
        self.unit(
            "torrents",
            "filePrio",
            Params::new()
                .str("hash", hash)
                .str(
                    "id",
                    ids.iter().map(i64::to_string).collect::<Vec<_>>().join("|"),
                )
                .int("priority", i64::from(prio)),
        )
        .await
    }
    pub async fn torrents_set_upload_limit(&self, h: &Hashes, limit: i64) -> Result<()> {
        self.unit(
            "torrents",
            "setUploadLimit",
            Params::new().hashes("hashes", h).int("limit", limit),
        )
        .await
    }
    pub async fn torrents_set_download_limit(&self, h: &Hashes, limit: i64) -> Result<()> {
        self.unit(
            "torrents",
            "setDownloadLimit",
            Params::new().hashes("hashes", h).int("limit", limit),
        )
        .await
    }
    pub async fn torrents_set_share_limits(
        &self,
        h: &Hashes,
        ratio: f64,
        seeding_time: i64,
        inactive: i64,
        mode: Option<&str>,
        action: Option<&str>,
    ) -> Result<()> {
        self.unit(
            "torrents",
            "setShareLimits",
            Params::new()
                .hashes("hashes", h)
                .str("ratioLimit", ratio.to_string())
                .int("seedingTimeLimit", seeding_time)
                .int("inactiveSeedingTimeLimit", inactive)
                .opt_str("shareLimitsMode", mode)
                .opt_str("shareLimitAction", action),
        )
        .await
    }
    pub async fn torrents_set_super_seeding(&self, h: &Hashes, v: bool) -> Result<()> {
        self.unit(
            "torrents",
            "setSuperSeeding",
            Params::new().hashes("hashes", h).bool("value", v),
        )
        .await
    }
    pub async fn torrents_set_force_start(&self, h: &Hashes, v: bool) -> Result<()> {
        self.unit(
            "torrents",
            "setForceStart",
            Params::new().hashes("hashes", h).bool("value", v),
        )
        .await
    }
    pub async fn torrents_set_location(&self, h: &Hashes, location: &str) -> Result<()> {
        self.unit(
            "torrents",
            "setLocation",
            Params::new().hashes("hashes", h).str("location", location),
        )
        .await
    }
    pub async fn torrents_set_save_path(&self, h: &Hashes, path: &str) -> Result<()> {
        self.unit(
            "torrents",
            "setSavePath",
            Params::new().hashes("id", h).str("path", path),
        )
        .await
    }
    pub async fn torrents_set_download_path(&self, h: &Hashes, path: &str) -> Result<()> {
        self.unit(
            "torrents",
            "setDownloadPath",
            Params::new().hashes("id", h).str("path", path),
        )
        .await
    }
    pub async fn torrents_set_auto_management(&self, h: &Hashes, enable: bool) -> Result<()> {
        self.unit(
            "torrents",
            "setAutoManagement",
            Params::new().hashes("hashes", h).bool("enable", enable),
        )
        .await
    }
    pub async fn torrents_rename_file(&self, hash: &str, old: &str, new: &str) -> Result<()> {
        self.unit(
            "torrents",
            "renameFile",
            Params::new()
                .str("hash", hash)
                .str("oldPath", old)
                .str("newPath", new),
        )
        .await
    }
    pub async fn torrents_rename_folder(&self, hash: &str, old: &str, new: &str) -> Result<()> {
        self.unit(
            "torrents",
            "renameFolder",
            Params::new()
                .str("hash", hash)
                .str("oldPath", old)
                .str("newPath", new),
        )
        .await
    }
    pub async fn torrents_set_ssl_parameters(&self, hash: &str, p: &SslParameters) -> Result<()> {
        self.unit(
            "torrents",
            "setSSLParameters",
            Params::new()
                .str("hash", hash)
                .str(
                    "ssl_certificate",
                    p.ssl_certificate.clone().unwrap_or_default(),
                )
                .str(
                    "ssl_private_key",
                    p.ssl_private_key.clone().unwrap_or_default(),
                )
                .str("ssl_dh_params", p.ssl_dh_params.clone().unwrap_or_default()),
        )
        .await
    }
    /// Call repeatedly to poll; 202 (pending) is returned as `Ok(None)`.
    pub async fn torrents_fetch_metadata(
        &self,
        source: &str,
        downloader: Option<&str>,
    ) -> Result<Option<Value>> {
        let r = self
            .call(
                "torrents",
                "fetchMetadata",
                Params::new()
                    .str("source", source)
                    .opt_str("downloader", downloader),
            )
            .await?;
        Ok(if r.status.as_u16() == 202 {
            None
        } else {
            Some(r.json()?)
        })
    }
    pub async fn torrents_cancel_fetch_metadata(&self, source: &str) -> Result<()> {
        self.unit(
            "torrents",
            "cancelFetchMetadata",
            Params::new().str("source", source),
        )
        .await
    }
    /// Parse local .torrent files server-side; the result array follows request order.
    pub async fn torrents_parse_metadata(&self, files: &[(String, Vec<u8>)]) -> Result<Value> {
        self.call_multipart("torrents", "parseMetadata", || {
            let mut f = reqwest::multipart::Form::new();
            for (n, b) in files {
                f = f.part(
                    "torrents",
                    reqwest::multipart::Part::bytes(b.clone()).file_name(n.clone()),
                );
            }
            f
        })
        .await?
        .json()
    }

    // ---------------- torrentcreator ----------------
    /// `fields` use the server's key names (`sourcePath`, `format`, `pieceSize`, …). Returns the task ID.
    pub async fn creator_add_task(&self, fields: &[(&str, String)]) -> Result<String> {
        let mut p = Params::new();
        for (k, v) in fields {
            p = p.str(k, v.clone());
        }
        let v: Value = self.call("torrentcreator", "addTask", p).await?.json()?;
        Ok(v.get("taskID")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned())
    }
    pub async fn creator_status(&self, task: Option<&str>) -> Result<Value> {
        self.call(
            "torrentcreator",
            "status",
            Params::new().opt_str("taskID", task),
        )
        .await?
        .json()
    }
    pub async fn creator_torrent_file(&self, task: &str) -> Result<Vec<u8>> {
        Ok(self
            .call(
                "torrentcreator",
                "torrentFile",
                Params::new().str("taskID", task),
            )
            .await?
            .body)
    }
    pub async fn creator_delete_task(&self, task: &str) -> Result<()> {
        self.unit(
            "torrentcreator",
            "deleteTask",
            Params::new().str("taskID", task),
        )
        .await
    }

    // ---------------- rss ----------------
    pub async fn rss_add_folder(&self, path: &str) -> Result<()> {
        self.unit("rss", "addFolder", Params::new().str("path", path))
            .await
    }
    pub async fn rss_add_feed(
        &self,
        url: &str,
        path: &str,
        refresh_interval: Option<i64>,
    ) -> Result<()> {
        let mut p = Params::new().str("url", url).str("path", path);
        if let Some(r) = refresh_interval {
            p = p.int("refreshInterval", r);
        }
        self.unit("rss", "addFeed", p).await
    }
    pub async fn rss_set_feed_url(&self, path: &str, url: &str) -> Result<()> {
        self.unit(
            "rss",
            "setFeedURL",
            Params::new().str("path", path).str("url", url),
        )
        .await
    }
    pub async fn rss_set_feed_refresh_interval(&self, path: &str, secs: i64) -> Result<()> {
        self.unit(
            "rss",
            "setFeedRefreshInterval",
            Params::new().str("path", path).int("refreshInterval", secs),
        )
        .await
    }
    pub async fn rss_remove_item(&self, path: &str) -> Result<()> {
        self.unit("rss", "removeItem", Params::new().str("path", path))
            .await
    }
    pub async fn rss_move_item(&self, item: &str, dest: &str) -> Result<()> {
        self.unit(
            "rss",
            "moveItem",
            Params::new().str("itemPath", item).str("destPath", dest),
        )
        .await
    }
    pub async fn rss_items(&self, with_data: bool) -> Result<Value> {
        self.call("rss", "items", Params::new().bool("withData", with_data))
            .await?
            .json()
    }
    pub async fn rss_mark_as_read(&self, item: &str, article: Option<&str>) -> Result<()> {
        self.unit(
            "rss",
            "markAsRead",
            Params::new()
                .str("itemPath", item)
                .opt_str("articleId", article),
        )
        .await
    }
    pub async fn rss_refresh_item(&self, item: &str) -> Result<()> {
        self.unit("rss", "refreshItem", Params::new().str("itemPath", item))
            .await
    }
    pub async fn rss_set_rule(&self, name: &str, def: &Value) -> Result<()> {
        self.unit(
            "rss",
            "setRule",
            Params::new()
                .str("ruleName", name)
                .str("ruleDef", json_str(def)?),
        )
        .await
    }
    pub async fn rss_rename_rule(&self, name: &str, new: &str) -> Result<()> {
        self.unit(
            "rss",
            "renameRule",
            Params::new().str("ruleName", name).str("newRuleName", new),
        )
        .await
    }
    pub async fn rss_clone_rule(&self, src: &str, clone: &str) -> Result<()> {
        self.unit(
            "rss",
            "cloneRule",
            Params::new().str("sourceName", src).str("cloneName", clone),
        )
        .await
    }
    pub async fn rss_remove_rule(&self, name: &str) -> Result<()> {
        self.unit("rss", "removeRule", Params::new().str("ruleName", name))
            .await
    }
    pub async fn rss_rules(&self) -> Result<Value> {
        self.call("rss", "rules", Params::new()).await?.json()
    }
    pub async fn rss_matching_articles(&self, rule: &str) -> Result<Value> {
        self.call(
            "rss",
            "matchingArticles",
            Params::new().str("ruleName", rule),
        )
        .await?
        .json()
    }
    pub async fn rss_export_rules(&self) -> Result<Vec<u8>> {
        Ok(self.call("rss", "exportRules", Params::new()).await?.body)
    }
    pub async fn rss_import_rules(&self, name: &str, bytes: &[u8]) -> Result<()> {
        self.call_multipart("rss", "importRules", || {
            reqwest::multipart::Form::new().part(
                "rules",
                reqwest::multipart::Part::bytes(bytes.to_vec()).file_name(name.to_owned()),
            )
        })
        .await?;
        Ok(())
    }

    // ---------------- search ----------------
    pub async fn search_start(
        &self,
        pattern: &str,
        category: &str,
        plugins: &str,
    ) -> Result<SearchStart> {
        self.call(
            "search",
            "start",
            Params::new()
                .str("pattern", pattern)
                .str("category", category)
                .str("plugins", plugins),
        )
        .await?
        .json()
    }
    pub async fn search_stop(&self, id: i64) -> Result<()> {
        self.unit("search", "stop", Params::new().int("id", id))
            .await
    }
    pub async fn search_delete(&self, id: i64) -> Result<()> {
        self.unit("search", "delete", Params::new().int("id", id))
            .await
    }
    pub async fn search_status(&self, id: Option<i64>) -> Result<Vec<SearchStatus>> {
        let mut p = Params::new();
        if let Some(i) = id {
            p = p.int("id", i);
        }
        self.call("search", "status", p).await?.json()
    }
    pub async fn search_results(&self, id: i64, limit: i64, offset: i64) -> Result<SearchResults> {
        self.call(
            "search",
            "results",
            Params::new()
                .int("id", id)
                .int("limit", limit)
                .int("offset", offset),
        )
        .await?
        .json()
    }
    pub async fn search_download_torrent(&self, url: &str, plugin: &str) -> Result<()> {
        self.unit(
            "search",
            "downloadTorrent",
            Params::new()
                .str("torrentUrl", url)
                .str("pluginName", plugin),
        )
        .await
    }
    pub async fn search_plugins(&self) -> Result<Vec<SearchPlugin>> {
        self.call("search", "plugins", Params::new()).await?.json()
    }
    /// `sources` are URLs or **server-side** paths.
    pub async fn search_install_plugin(&self, sources: &[String]) -> Result<()> {
        self.unit(
            "search",
            "installPlugin",
            Params::new().str("sources", pipe(sources)),
        )
        .await
    }
    pub async fn search_uninstall_plugin(&self, names: &[String]) -> Result<()> {
        self.unit(
            "search",
            "uninstallPlugin",
            Params::new().str("names", pipe(names)),
        )
        .await
    }
    pub async fn search_enable_plugin(&self, names: &[String], enable: bool) -> Result<()> {
        self.unit(
            "search",
            "enablePlugin",
            Params::new()
                .str("names", pipe(names))
                .bool("enable", enable),
        )
        .await
    }
    pub async fn search_update_plugins(&self) -> Result<()> {
        self.unit("search", "updatePlugins", Params::new()).await
    }

    // ---------------- clientdata ----------------
    pub async fn clientdata_load(&self, keys: &[String]) -> Result<Value> {
        self.call(
            "clientdata",
            "load",
            Params::new().str("keys", json_str(&keys)?),
        )
        .await?
        .json()
    }
    pub async fn clientdata_store(&self, data: &Map<String, Value>) -> Result<()> {
        self.unit(
            "clientdata",
            "store",
            Params::new().str("data", json_str(data)?),
        )
        .await
    }
}

fn parse_i64(s: String) -> Result<i64> {
    s.trim()
        .parse()
        .map_err(|_| Error::Decode(format!("expected integer, got {s:?}")))
}
