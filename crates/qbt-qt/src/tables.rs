//! Conversions from API data to flat table rows for [`crate::model::GenericTable`]. Qt-free.
//!
//! Each `source` has a fixed column order documented next to its builder; QML relies on it.

use qbt_core::{Data, fmt};
use qbt_sync::{Counts, MainDataStore, StatusFilter};
use serde_json::{Map, Value, json};

pub type Row = Vec<Value>;

#[derive(Debug, Clone, Default)]
pub struct Payload {
    pub keys: Vec<String>,
    pub rows: Vec<Row>,
    /// Append to existing rows instead of replacing them (logs, paged search results).
    pub append: bool,
}

impl Payload {
    fn push(&mut self, key: impl Into<String>, row: Row) {
        self.keys.push(key.into());
        self.rows.push(row);
    }
}

fn s(v: &Option<String>) -> Value {
    Value::String(v.clone().unwrap_or_default())
}
fn n(v: Option<i64>) -> Value {
    v.map_or(Value::Null, Value::from)
}

/// Sidebar: `kind, key, label, count, depth`. `kind` is `header`, `status`, `category`, `tag` or `tracker`.
pub fn sidebar(counts: &Counts, store: &MainDataStore) -> Payload {
    let mut p = Payload::default();
    let add =
        |p: &mut Payload, kind: &str, key: &str, label: &str, count: Option<usize>, depth: i64| {
            p.push(
                format!("{kind}:{key}"),
                vec![
                    json!(kind),
                    json!(key),
                    json!(label),
                    count.map_or(Value::Null, |c| json!(c)),
                    json!(depth),
                ],
            );
        };
    add(&mut p, "header", "status", "Status", None, 0);
    for f in StatusFilter::ALL {
        let label = match f {
            StatusFilter::All => "All",
            StatusFilter::Downloading => "Downloading",
            StatusFilter::Seeding => "Seeding",
            StatusFilter::Completed => "Completed",
            StatusFilter::Stopped => "Stopped",
            StatusFilter::Running => "Running",
            StatusFilter::Active => "Active",
            StatusFilter::Inactive => "Inactive",
            StatusFilter::Stalled => "Stalled",
            StatusFilter::StalledUploading => "Stalled Uploading",
            StatusFilter::StalledDownloading => "Stalled Downloading",
            StatusFilter::Checking => "Checking",
            StatusFilter::Moving => "Moving",
            StatusFilter::Errored => "Errored",
        };
        add(
            &mut p,
            "status",
            f.api_name(),
            label,
            Some(counts.status.get(&f).copied().unwrap_or(0)),
            1,
        );
    }
    add(&mut p, "header", "category", "Categories", None, 0);
    add(&mut p, "category", "", "All", Some(counts.total), 1);
    add(
        &mut p,
        "category",
        "\u{0}uncategorized",
        "Uncategorized",
        Some(counts.uncategorized),
        1,
    );
    let mut names: Vec<&String> = store.categories.keys().collect();
    for k in counts.categories.keys() {
        if !names.contains(&k) {
            names.push(k);
        }
    }
    names.sort_by_key(|x| x.to_lowercase());
    for name in names {
        let depth = name.matches('/').count() as i64 + 1;
        let label = name.rsplit('/').next().unwrap_or(name);
        add(
            &mut p,
            "category",
            name,
            label,
            Some(counts.categories.get(name.as_str()).copied().unwrap_or(0)),
            depth,
        );
    }
    add(&mut p, "header", "tag", "Tags", None, 0);
    add(&mut p, "tag", "", "All", Some(counts.total), 1);
    add(
        &mut p,
        "tag",
        "\u{0}untagged",
        "Untagged",
        Some(counts.untagged),
        1,
    );
    let mut tags: Vec<&String> = store.tags.iter().collect();
    tags.sort_by_key(|x| x.to_lowercase());
    for t in tags {
        add(
            &mut p,
            "tag",
            t,
            t,
            Some(counts.tags.get(t.as_str()).copied().unwrap_or(0)),
            1,
        );
    }
    add(&mut p, "header", "tracker", "Trackers", None, 0);
    add(&mut p, "tracker", "", "All", Some(counts.total), 1);
    add(
        &mut p,
        "tracker",
        "\u{0}trackerless",
        "Trackerless",
        Some(counts.trackerless),
        1,
    );
    add(
        &mut p,
        "tracker",
        "\u{0}error",
        "Error",
        Some(counts.tracker_error),
        1,
    );
    add(
        &mut p,
        "tracker",
        "\u{0}warning",
        "Warning",
        Some(counts.tracker_warning),
        1,
    );
    add(
        &mut p,
        "tracker",
        "\u{0}other",
        "Other error",
        Some(counts.tracker_other_error),
        1,
    );
    for (h, c) in &counts.tracker_hosts {
        add(&mut p, "tracker", h, h, Some(*c), 1);
    }
    p
}

/// Single-column name lists for menus: category names and tag names.
pub fn name_lists(store: &MainDataStore, counts: &Counts) -> (Payload, Payload) {
    let mut cats: Vec<&String> = store.categories.keys().collect();
    for k in counts.categories.keys() {
        if !cats.contains(&k) {
            cats.push(k);
        }
    }
    cats.sort_by_key(|x| x.to_lowercase());
    let mut c = Payload::default();
    for k in cats {
        c.push(k.clone(), vec![json!(k)]);
    }
    let mut tags: Vec<&String> = store.tags.iter().collect();
    tags.sort_by_key(|x| x.to_lowercase());
    let mut t = Payload::default();
    for k in tags {
        t.push(k.clone(), vec![json!(k)]);
    }
    (c, t)
}

/// Peers: `ip:port, client, connection, flags, country, progress, dl_speed, up_speed, downloaded, uploaded, relevance, files, host`.
pub const PEER_COLS: usize = 13;
pub fn peers_row(key: &str, o: &Map<String, Value>) -> Row {
    let g = |k: &str| o.get(k).cloned().unwrap_or(Value::Null);
    let i = |k: &str| o.get(k).and_then(Value::as_i64).unwrap_or(0);
    let addr = match (
        o.get("ip").and_then(Value::as_str),
        o.get("port").and_then(Value::as_i64),
    ) {
        (Some(ip), Some(p)) if ip.contains(':') => format!("[{ip}]:{p}"),
        (Some(ip), Some(p)) => format!("{ip}:{p}"),
        _ => o
            .get("i2p_dest")
            .and_then(Value::as_str)
            .unwrap_or(key)
            .to_owned(),
    };
    vec![
        json!(addr),
        g("client"),
        g("connection"),
        g("flags"),
        json!(
            format!(
                "{} {}",
                flag_emoji(o.get("country_code").and_then(Value::as_str).unwrap_or("")),
                o.get("country").and_then(Value::as_str).unwrap_or("")
            )
            .trim()
        ),
        json!(
            o.get("progress")
                .and_then(Value::as_f64)
                .map(fmt::percent)
                .unwrap_or_default()
        ),
        json!(fmt::speed_or_empty(i("dl_speed"))),
        json!(fmt::speed_or_empty(i("up_speed"))),
        json!(fmt::size(i("downloaded"))),
        json!(fmt::size(i("uploaded"))),
        json!(
            o.get("relevance")
                .and_then(Value::as_f64)
                .map(|r| format!("{:.1}%", r * 100.0))
                .unwrap_or_default()
        ),
        g("files"),
        g("host_name"),
    ]
}

/// Regional-indicator emoji for an ISO country code.
pub fn flag_emoji(cc: &str) -> String {
    if cc.len() != 2 || !cc.is_ascii() {
        return String::new();
    }
    cc.to_ascii_uppercase()
        .chars()
        .filter_map(|c| char::from_u32(0x1F1E6 + (c as u32 - 'A' as u32)))
        .collect()
}

/// Trackers: `url_or_endpoint, status, tier, peers, seeds, leeches, downloaded, message, next_announce, depth`.
pub fn trackers(list: &[qbt_api::models::torrents::Tracker]) -> Payload {
    use qbt_api::models::TrackerStatus as S;
    let status = |s: &Option<S>| match s {
        Some(S::Disabled) => "Disabled",
        Some(S::NotContacted) => "Not contacted yet",
        Some(S::Working) => "Working",
        Some(S::Updating) => "Updating...",
        Some(S::NotWorking) => "Not working",
        Some(S::TrackerError) => "Tracker error",
        Some(S::Unreachable) => "Unreachable",
        _ => "",
    };
    let mut p = Payload::default();
    for (ti, t) in list.iter().enumerate() {
        p.push(
            format!("t{ti}"),
            vec![
                s(&t.url),
                json!(status(&t.status)),
                n(t.tier),
                n(t.num_peers),
                n(t.num_seeds),
                n(t.num_leeches),
                n(t.num_downloaded),
                s(&t.msg),
                json!(t.next_announce.map(fmt::duration).unwrap_or_default()),
                json!(0),
            ],
        );
        for (ei, e) in t.endpoints.iter().flatten().enumerate() {
            p.push(
                format!("t{ti}e{ei}"),
                vec![
                    s(&e.name),
                    json!(status(&e.status)),
                    Value::Null,
                    n(e.num_peers),
                    n(e.num_seeds),
                    n(e.num_leeches),
                    n(e.num_downloaded),
                    s(&e.msg),
                    json!(e.next_announce.map(fmt::duration).unwrap_or_default()),
                    json!(1),
                ],
            );
        }
    }
    p
}

/// Web seeds: `url`.
pub fn webseeds(list: &[qbt_api::models::torrents::WebSeed]) -> Payload {
    let mut p = Payload::default();
    for w in list {
        p.push(w.url.clone().unwrap_or_default(), vec![s(&w.url)]);
    }
    p
}

/// Content tree flattened depth-first: `name, size, progress, priority, availability, index, path, depth, is_folder, pieces`.
/// Folder rows aggregate size, progress and priority (`-1` = mixed).
pub fn files(list: &[qbt_api::models::torrents::TorrentFile]) -> Payload {
    #[derive(Default)]
    struct Node {
        size: i64,
        done: f64,
        prio: Option<i64>,
        mixed: bool,
        children: std::collections::BTreeMap<String, Node>,
        file: Option<usize>,
    }
    let mut root = Node::default();
    for (idx, f) in list.iter().enumerate() {
        let name = f.name.clone().unwrap_or_default();
        let (size, prog, prio) = (
            f.size.unwrap_or(0),
            f.progress.unwrap_or(0.0),
            f.priority.map(i64::from).unwrap_or(1),
        );
        let mut node = &mut root;
        for part in name.split('/') {
            node.size += size;
            node.done += size as f64 * prog;
            match node.prio {
                None => node.prio = Some(prio),
                Some(x) if x != prio => node.mixed = true,
                _ => {}
            }
            node = node.children.entry(part.to_owned()).or_default();
        }
        node.size = size;
        node.done = size as f64 * prog;
        node.prio = Some(prio);
        node.file = Some(idx);
    }
    fn walk(
        node: &Node,
        prefix: &str,
        depth: i64,
        list: &[qbt_api::models::torrents::TorrentFile],
        p: &mut Payload,
    ) {
        for (name, ch) in &node.children {
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            let is_folder = ch.file.is_none();
            let prog = if ch.size > 0 {
                ch.done / ch.size as f64
            } else {
                0.0
            };
            let prio = if ch.mixed { -1 } else { ch.prio.unwrap_or(1) };
            let (avail, pieces) =
                ch.file
                    .and_then(|i| list.get(i))
                    .map_or((Value::Null, Value::Null), |f| {
                        (
                            f.availability
                                .map_or(Value::Null, |a| json!(format!("{a:.3}"))),
                            json!(
                                f.piece_range
                                    .as_ref()
                                    .map(|r| r
                                        .iter()
                                        .map(i64::to_string)
                                        .collect::<Vec<_>>()
                                        .join("-"))
                                    .unwrap_or_default()
                            ),
                        )
                    });
            p.push(
                path.clone(),
                vec![
                    json!(name),
                    json!(fmt::size(ch.size)),
                    json!(fmt::percent(prog)),
                    json!(prio),
                    avail,
                    json!(ch.file.map_or(-1, |i| list[i].index.unwrap_or(i as i64))),
                    json!(path),
                    json!(depth),
                    json!(is_folder),
                    pieces,
                ],
            );
            walk(ch, &path, depth + 1, list, p);
        }
    }
    let mut p = Payload::default();
    walk(&root, "", 0, list, &mut p);
    p
}

/// Main log: `id, time, type, message`. `type` is 1 normal, 2 info, 4 warning, 8 critical.
pub fn main_log(list: &[qbt_api::models::log::LogEntry]) -> Payload {
    let mut p = Payload {
        append: true,
        ..Default::default()
    };
    for e in list {
        p.push(
            format!("l{}", e.id.unwrap_or(0)),
            vec![
                n(e.id),
                json!(fmt::datetime(e.timestamp.unwrap_or(0) / 1000)),
                n(e.kind),
                s(&e.message),
            ],
        );
    }
    p
}

/// Peer log: `id, time, ip, blocked, reason`.
pub fn peer_log(list: &[qbt_api::models::log::PeerLogEntry]) -> Payload {
    let mut p = Payload {
        append: true,
        ..Default::default()
    };
    for e in list {
        p.push(
            format!("p{}", e.id.unwrap_or(0)),
            vec![
                n(e.id),
                json!(fmt::datetime(e.timestamp.unwrap_or(0) / 1000)),
                s(&e.ip),
                json!(e.blocked.unwrap_or(false)),
                s(&e.reason),
            ],
        );
    }
    p
}

/// Search results: `name, size, seeders, leechers, engine, site, file_url, descr_link, pub_date`.
pub fn search_results(offset: i64, list: &[qbt_api::models::search::SearchResult]) -> Payload {
    let mut p = Payload {
        append: true,
        ..Default::default()
    };
    for (i, r) in list.iter().enumerate() {
        p.push(
            format!("r{}", offset + i as i64),
            vec![
                s(&r.file_name),
                n(r.file_size),
                n(r.nb_seeders),
                n(r.nb_leechers),
                s(&r.engine_name),
                s(&r.site_url),
                s(&r.file_url),
                s(&r.descr_link),
                json!(fmt::datetime(r.pub_date.unwrap_or(0))),
            ],
        );
    }
    p
}

/// Search plugins: `name, full_name, version, enabled, url`.
pub fn plugins(list: &[qbt_api::models::search::SearchPlugin]) -> Payload {
    let mut p = Payload::default();
    for x in list {
        p.push(
            x.name.clone().unwrap_or_default(),
            vec![
                s(&x.name),
                s(&x.full_name),
                s(&x.version),
                json!(x.enabled.unwrap_or(false)),
                s(&x.url),
            ],
        );
    }
    p
}

/// Torrent-creator tasks: `taskID, source, status, progress, error, added`.
pub fn creator_tasks(v: &Value) -> Payload {
    let mut p = Payload::default();
    for t in v.as_array().into_iter().flatten() {
        let g = |k: &str| t.get(k).cloned().unwrap_or(Value::Null);
        let id = t
            .get("taskID")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        p.push(
            id,
            vec![
                g("taskID"),
                g("sourcePath"),
                g("status"),
                json!(
                    t.get("progress")
                        .and_then(Value::as_f64)
                        .map(|x| fmt::percent(x / 100.0))
                        .unwrap_or_default()
                ),
                g("errorMessage"),
                json!(fmt::datetime(
                    t.get("timeAdded").and_then(Value::as_i64).unwrap_or(0)
                )),
            ],
        );
    }
    p
}

/// RSS tree flattened: `path, name, kind(feed|folder), unread, url, depth`.
/// Articles are in [`rss_articles`].
pub fn rss_tree(v: &Value) -> Payload {
    fn unread(node: &Value) -> i64 {
        match node.get("articles").and_then(Value::as_array) {
            Some(a) => a
                .iter()
                .filter(|x| !x.get("isRead").and_then(Value::as_bool).unwrap_or(false))
                .count() as i64,
            None => node.as_object().map_or(0, |o| {
                o.values().filter(|c| c.is_object()).map(unread).sum()
            }),
        }
    }
    fn walk(obj: &Map<String, Value>, prefix: &str, depth: i64, p: &mut Payload) {
        let mut names: Vec<_> = obj.keys().collect();
        names.sort_by_key(|x| x.to_lowercase());
        for name in names {
            let node = &obj[name];
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}\\{name}")
            };
            let is_feed = node.get("uid").is_some() || node.get("url").is_some();
            p.push(
                path.clone(),
                vec![
                    json!(path),
                    json!(name),
                    json!(if is_feed { "feed" } else { "folder" }),
                    json!(unread(node)),
                    node.get("url").cloned().unwrap_or(Value::Null),
                    json!(depth),
                ],
            );
            if !is_feed && let Some(o) = node.as_object() {
                walk(o, &path, depth + 1, p);
            }
        }
    }
    let mut p = Payload::default();
    if let Some(o) = v.as_object() {
        walk(o, "", 0, &mut p);
    }
    p
}

/// Articles of one feed or folder: `title, date, read, id, link, description, feed_path, torrent_url`.
pub fn rss_articles(v: &Value, selected: &str) -> Payload {
    fn collect(node: &Value, path: &str, sel: &str, out: &mut Payload) {
        if let Some(a) = node.get("articles").and_then(Value::as_array) {
            if sel.is_empty() || path == sel || path.starts_with(&format!("{sel}\\")) {
                for art in a {
                    let g = |k: &str| art.get(k).cloned().unwrap_or(Value::Null);
                    let id = art
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_owned();
                    out.push(
                        format!("{path}/{id}"),
                        vec![
                            g("title"),
                            g("date"),
                            g("isRead"),
                            g("id"),
                            g("link"),
                            g("description"),
                            json!(path),
                            art.get("torrentURL").cloned().unwrap_or_else(|| g("link")),
                        ],
                    );
                }
            }
        } else if let Some(o) = node.as_object() {
            for (k, c) in o {
                let cp = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}\\{k}")
                };
                collect(c, &cp, sel, out);
            }
        }
    }
    let mut p = Payload::default();
    collect(v, "", selected, &mut p);
    p
}

/// RSS rules: `name, enabled`.
pub fn rss_rules(v: &Value) -> Payload {
    let mut p = Payload::default();
    if let Some(o) = v.as_object() {
        for (k, r) in o {
            p.push(
                k.clone(),
                vec![json!(k), r.get("enabled").cloned().unwrap_or(json!(true))],
            );
        }
    }
    p
}

/// Remote directory listing: `name, type, size, path`.
pub fn dir_listing(parent: &str, v: &Value) -> Payload {
    let mut p = Payload::default();
    let sep = if parent.contains('\\') && !parent.contains('/') {
        "\\"
    } else {
        "/"
    };
    for e in v.as_array().into_iter().flatten() {
        let name = e.get("name").and_then(Value::as_str).unwrap_or("");
        let path = if parent.ends_with(['/', '\\']) {
            format!("{parent}{name}")
        } else {
            format!("{parent}{sep}{name}")
        };
        p.push(
            path.clone(),
            vec![
                json!(name),
                e.get("type").cloned().unwrap_or(Value::Null),
                e.get("size").cloned().unwrap_or(Value::Null),
                json!(path),
            ],
        );
    }
    p.rows.sort_by(|a, b| {
        let dir = |r: &Row| r[1] == json!("dir");
        dir(b).cmp(&dir(a)).then_with(|| {
            a[0].as_str()
                .unwrap_or("")
                .to_lowercase()
                .cmp(&b[0].as_str().unwrap_or("").to_lowercase())
        })
    });
    // Keep keys aligned with the sorted rows.
    p.keys = p
        .rows
        .iter()
        .map(|r| r[3].as_str().unwrap_or("").to_owned())
        .collect();
    p
}

/// Categories: `name, save_path`.
pub fn categories(v: &Value) -> Payload {
    let mut p = Payload::default();
    if let Some(o) = v.as_object() {
        for (k, c) in o {
            p.push(
                k.clone(),
                vec![json!(k), c.get("savePath").cloned().unwrap_or(Value::Null)],
            );
        }
    }
    p
}

/// Cookies: `name, domain, path, value, expiration`.
pub fn cookies(v: &Value) -> Payload {
    let mut p = Payload::default();
    for (i, c) in v.as_array().into_iter().flatten().enumerate() {
        let g = |k: &str| c.get(k).cloned().unwrap_or(Value::Null);
        p.push(
            format!("c{i}"),
            vec![
                g("name"),
                g("domain"),
                g("path"),
                g("value"),
                json!(fmt::datetime(
                    c.get("expirationDate").and_then(Value::as_i64).unwrap_or(0)
                )),
            ],
        );
    }
    p
}

/// Profiles: `name, url, auth`.
pub fn profiles(list: &[qbt_core::Profile]) -> Payload {
    let mut p = Payload::default();
    for x in list {
        p.push(
            x.name.clone(),
            vec![
                json!(x.name),
                json!(x.url),
                json!(format!("{:?}", x.auth).to_lowercase()),
            ],
        );
    }
    p
}

/// General tab: `label, value` pairs from `torrents/properties`.
pub fn general(p: &qbt_api::models::torrents::TorrentProperties) -> Payload {
    let mut out = Payload::default();
    let mut add = |label: &str, v: String| out.push(label.to_owned(), vec![json!(label), json!(v)]);
    let i = |x: Option<i64>| x.unwrap_or(0);
    add("Save Path", p.save_path.clone().unwrap_or_default());
    add("Total Size", fmt::size(i(p.total_size)));
    add(
        "Pieces",
        format!(
            "{} x {} (have {})",
            i(p.pieces_num),
            fmt::size(i(p.piece_size)),
            i(p.pieces_have)
        ),
    );
    add("Added On", fmt::datetime(i(p.addition_date)));
    add("Completed On", fmt::datetime(i(p.completion_date)));
    add("Created By", p.created_by.clone().unwrap_or_default());
    add("Created On", fmt::datetime(i(p.creation_date)));
    add("Last Seen Complete", fmt::datetime(i(p.last_seen)));
    add("Time Active", fmt::duration(i(p.time_elapsed)));
    add("Seeding Time", fmt::duration(i(p.seeding_time)));
    add("ETA", fmt::eta(i(p.eta)));
    add(
        "Connections",
        format!(
            "{} ({} max)",
            i(p.nb_connections),
            i(p.nb_connections_limit)
        ),
    );
    add(
        "Downloaded",
        format!(
            "{} ({} this session)",
            fmt::size(i(p.total_downloaded)),
            fmt::size(i(p.total_downloaded_session))
        ),
    );
    add(
        "Uploaded",
        format!(
            "{} ({} this session)",
            fmt::size(i(p.total_uploaded)),
            fmt::size(i(p.total_uploaded_session))
        ),
    );
    add("Wasted", fmt::size(i(p.total_wasted)));
    add(
        "Download Speed",
        format!(
            "{} (avg {})",
            fmt::speed(i(p.dl_speed)),
            fmt::speed(i(p.dl_speed_avg))
        ),
    );
    add(
        "Upload Speed",
        format!(
            "{} (avg {})",
            fmt::speed(i(p.up_speed)),
            fmt::speed(i(p.up_speed_avg))
        ),
    );
    add("Download Limit", fmt::limit(i(p.dl_limit)));
    add("Upload Limit", fmt::limit(i(p.up_limit)));
    add(
        "Seeds",
        format!("{} ({} total)", i(p.seeds), i(p.seeds_total)),
    );
    add(
        "Peers",
        format!("{} ({} total)", i(p.peers), i(p.peers_total)),
    );
    add("Share Ratio", fmt::ratio(p.share_ratio.unwrap_or(0.0)));
    add("Reannounce In", fmt::duration(i(p.reannounce)));
    add("Info Hash v1", p.infohash_v1.clone().unwrap_or_default());
    add("Info Hash v2", p.infohash_v2.clone().unwrap_or_default());
    add("Comment", p.comment.clone().unwrap_or_default());
    add(
        "Private",
        if p.private.or(p.is_private).unwrap_or(false) {
            "Yes".into()
        } else {
            "No".into()
        },
    );
    out
}

/// Search jobs: `id, status, total`.
pub fn search_jobs(list: &[qbt_api::models::search::SearchStatus]) -> Payload {
    let mut p = Payload::default();
    for j in list {
        p.push(
            format!("{}", j.id.unwrap_or(0)),
            vec![n(j.id), s(&j.status), n(j.total)],
        );
    }
    p
}

/// Add-dialog preview: tolerant extraction of the file list from `parseMetadata`/`fetchMetadata`
/// output (first torrent when an array). Same columns as [`files`].
pub fn preview_from_meta(v: &Value) -> Payload {
    let first = match v {
        Value::Array(a) => a.first().unwrap_or(&Value::Null),
        o => o,
    };
    let info = first.get("info").unwrap_or(first);
    let list: Vec<qbt_api::models::torrents::TorrentFile> = info
        .get("files")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .enumerate()
                .map(|(i, f)| {
                    let name = f
                        .get("name")
                        .or_else(|| f.get("path"))
                        .and_then(|n| match n {
                            Value::String(s) => Some(s.clone()),
                            Value::Array(parts) => Some(
                                parts
                                    .iter()
                                    .filter_map(Value::as_str)
                                    .collect::<Vec<_>>()
                                    .join("/"),
                            ),
                            _ => None,
                        });
                    qbt_api::models::torrents::TorrentFile {
                        index: Some(i as i64),
                        name,
                        size: f
                            .get("size")
                            .or_else(|| f.get("length"))
                            .and_then(Value::as_i64),
                        progress: Some(0.0),
                        // The server marks files matching its excluded-file-names filter with priority 0.
                        priority: Some(qbt_api::models::FilePriority::from(
                            f.get("priority").and_then(Value::as_i64).unwrap_or(1),
                        )),
                        ..Default::default()
                    }
                })
                .collect()
        })
        .unwrap_or_else(|| {
            // Single-file torrent: name and size at the top level.
            match (
                info.get("name").and_then(Value::as_str),
                info.get("size")
                    .or_else(|| info.get("length"))
                    .and_then(Value::as_i64),
            ) {
                (Some(n), sz) => vec![qbt_api::models::torrents::TorrentFile {
                    index: Some(0),
                    name: Some(n.into()),
                    size: sz,
                    progress: Some(0.0),
                    priority: Some(qbt_api::models::FilePriority::Normal),
                    ..Default::default()
                }],
                _ => vec![],
            }
        });
    files(&list)
}

/// Convert a [`Data`] event to its table payload, if it feeds one.
pub fn from_data(d: &Data) -> Option<(&'static str, Payload)> {
    Some(match d {
        Data::Trackers(t) => ("trackers", trackers(t)),
        Data::WebSeeds(w) => ("webseeds", webseeds(w)),
        Data::Files(f) => ("files", files(f)),
        Data::MainLog(l) => ("log", main_log(l)),
        Data::PeerLog(l) => ("peerlog", peer_log(l)),
        Data::SearchResults {
            offset, results, ..
        } => ("search_results", search_results(*offset, results)),
        Data::CreatorStatus(v) => ("creator", creator_tasks(v)),
        Data::Properties(p) => ("general", general(p)),
        Data::SearchStatus(l) => ("search_jobs", search_jobs(l)),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use qbt_api::models::torrents::{TorrentFile, Tracker, TrackerEndpoint};

    #[test]
    fn files_become_a_tree_with_aggregates() {
        let f = |name: &str, size: i64, prog: f64, prio: i64| TorrentFile {
            name: Some(name.into()),
            size: Some(size),
            progress: Some(prog),
            priority: Some(prio.into()),
            ..Default::default()
        };
        let p = files(&[
            f("d/a.txt", 100, 1.0, 1),
            f("d/b.txt", 100, 0.0, 0),
            f("top.bin", 50, 0.5, 7),
        ]);
        assert_eq!(p.keys, ["d", "d/a.txt", "d/b.txt", "top.bin"]);
        assert_eq!(p.rows[0][7], json!(0));
        assert_eq!(p.rows[0][8], json!(true));
        assert_eq!(p.rows[0][3], json!(-1), "mixed priority");
        assert_eq!(p.rows[0][2], json!("50.0%"));
        assert_eq!(p.rows[1][7], json!(1));
        assert_eq!(p.rows[3][3], json!(7));
    }

    #[test]
    fn trackers_nest_endpoints() {
        let t = Tracker {
            url: Some("http://t/a".into()),
            status: Some(2.into()),
            endpoints: Some(vec![TrackerEndpoint {
                name: Some("http://1.2.3.4".into()),
                ..Default::default()
            }]),
            ..Default::default()
        };
        let p = trackers(&[t]);
        assert_eq!(p.keys, ["t0", "t0e0"]);
        assert_eq!(
            (p.rows[0][9].clone(), p.rows[1][9].clone()),
            (json!(0), json!(1))
        );
        assert_eq!(p.rows[0][1], json!("Working"));
    }

    #[test]
    fn peer_rows_and_flags() {
        let o = json!({"ip":"::1","port":6881,"client":"qB","country_code":"de","country":"Germany","progress":1.0,"dl_speed":0,"up_speed":1024}).as_object().unwrap().clone();
        let r = peers_row("::1:6881", &o);
        assert_eq!(r.len(), PEER_COLS);
        assert_eq!(r[0], json!("[::1]:6881"));
        assert_eq!(r[4], json!("🇩🇪 Germany"));
        assert_eq!(flag_emoji("zz1"), "");
    }

    #[test]
    fn sidebar_has_sections_and_nested_categories() {
        let mut s = MainDataStore::default();
        s.apply(&serde_json::from_value(json!({"rid":1,"full_update":true,"torrents":{"a":{"state":"downloading","category":"tv/shows","tags":"x"}},
            "categories":{"tv":{},"tv/shows":{}},"tags":["x"],"trackers":{"http://h.org/a":["a"]}})).unwrap());
        let c = Counts::compute(&s);
        let p = sidebar(&c, &s);
        let find = |k: &str| {
            p.keys
                .iter()
                .position(|x| x == k)
                .unwrap_or_else(|| panic!("{k}"))
        };
        assert_eq!(p.rows[find("category:tv/shows")][4], json!(2));
        assert_eq!(p.rows[find("category:tv")][3], json!(1));
        assert_eq!(p.rows[find("tag:x")][3], json!(1));
        assert_eq!(p.rows[find("tracker:h.org")][3], json!(1));
        assert_eq!(p.rows[find("status:downloading")][3], json!(1));
    }

    #[test]
    fn rss_flattening() {
        let v = json!({"Folder":{"Feed1":{"uid":"x","url":"http://f","articles":[{"id":"1","title":"T","isRead":false},{"id":"2","title":"U","isRead":true}]}}});
        let t = rss_tree(&v);
        assert_eq!(t.keys, ["Folder", "Folder\\Feed1"]);
        assert_eq!(t.rows[0][3], json!(1));
        assert_eq!(t.rows[1][2], json!("feed"));
        assert_eq!(rss_articles(&v, "Folder\\Feed1").rows.len(), 2);
        assert_eq!(rss_articles(&v, "").rows.len(), 2);
        assert_eq!(rss_articles(&v, "Other").rows.len(), 0);
    }

    #[test]
    fn preview_handles_file_lists_and_single_files() {
        let p = preview_from_meta(
            &json!([{"info":{"files":[{"path":["d","a.txt"],"length":10},{"name":"b","size":5}]}}]),
        );
        assert_eq!(p.keys, ["b", "d", "d/a.txt"]);
        let p = preview_from_meta(&json!({"name":"one.iso","size":7}));
        assert_eq!(p.keys, ["one.iso"]);
        assert!(preview_from_meta(&json!(null)).keys.is_empty());
        // Files excluded by the server's filter arrive with priority 0.
        let p = preview_from_meta(
            &json!({"info":{"files":[{"path":"a.nfo","length":1,"priority":0},{"path":"b.mkv","length":2}]}}),
        );
        assert_eq!(
            (p.rows[0][3].clone(), p.rows[1][3].clone()),
            (json!(0), json!(1))
        );
    }

    #[test]
    fn dir_listing_sorts_dirs_first() {
        let p = dir_listing(
            "/data",
            &json!([{"name":"b.txt","type":"file"},{"name":"A","type":"dir"}]),
        );
        assert_eq!(p.keys, ["/data/A", "/data/b.txt"]);
    }
}
