#!/usr/bin/env python3
"""A small stand-in for a qBittorrent Web UI (WebAPI 2.17.0) for developing and testing the client UI.

Not a faithful implementation: it serves plausible data for the endpoints the UI polls and accepts
every mutation. Torrent progress and speeds change over time so `sync/maindata` produces deltas.

    python3 tools/mock_qbt.py [--port 8080] [--api-key KEY] [--user admin --password adminadmin]
"""
import argparse
import json
import random
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

STATE_LOCK = threading.Lock()
RID = 0
TORRENTS = {}
CHANGES = {}      # rid -> {hash: {field: value}}
REMOVED = {}      # rid -> [hash]
PENDING = {}      # mutations made by API calls, folded into the next tick's delta
PENDING_REMOVED = []
LOG = []
PREFS = {
    "save_path": "/downloads", "web_ui_port": 8080, "dl_limit": 0, "up_limit": 0, "dht": True,
    "queueing_enabled": True, "max_active_downloads": 3, "locale": "en", "web_ui_api_key": "SECRET-KEY",
    "proxy_password": "hunter2", "torrent_content_layout": "Original", "add_stopped_enabled": False,
    "listen_port": 6881, "max_ratio": -1.0, "delete_torrent_content_files": False, "confirm_torrent_deletion": True,
}
CATEGORIES = {"tv": {"name": "tv", "savePath": "/downloads/tv"}, "tv/shows": {"name": "tv/shows", "savePath": ""},
              "movies": {"name": "movies", "savePath": "/downloads/movies"}}
TAGS = ["hd", "linux"]
SERVER = {"alt": False, "paused": False}
TOKENS = set()
ARGS = None
POST_ONLY = set()


def load_methods():
    """POST-only actions, from the generated docs/endpoints.toml (so the mock enforces 405 like the real server)."""
    import os, re
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "docs", "endpoints.toml")
    try:
        text = open(path).read()
    except OSError:
        return
    for block in text.split("[[endpoint]]")[1:]:
        f = dict(re.findall(r'(\w+) = "([^"]*)"', block))
        if f.get("method") == "POST":
            POST_ONLY.add((f["scope"], f["action"]))


def matches_filter(t, f):
    """Status filters as implemented upstream (torrentfilter.cpp / torrentimpl.cpp), written independently of the client."""
    st = t["state"]
    stopped = st in ("stoppedUP", "stoppedDL")
    down = st in ("downloading", "metaDL", "forcedMetaDL", "stalledDL", "checkingDL", "stoppedDL", "queuedDL", "forcedDL")
    up = st in ("uploading", "stalledUP", "checkingUP", "queuedUP", "forcedUP")
    active = t["dlspeed"] > 0 or t["upspeed"] > 0
    return {"all": True, "downloading": down, "seeding": up, "completed": up or st == "stoppedUP", "stopped": stopped,
            "running": not stopped, "active": active, "inactive": not active, "stalled": st in ("stalledUP", "stalledDL"),
            "stalled_uploading": st == "stalledUP", "stalled_downloading": st == "stalledDL",
            "checking": st in ("checkingUP", "checkingDL", "checkingResumeData"), "moving": st == "moving",
            "errored": st in ("error", "missingFiles")}.get(f, True)


def torrents_info(q):
    with STATE_LOCK:
        f = q.get("filter", "all")
        return [dict(public(t), hash=h) for h, t in TORRENTS.items() if matches_filter(t, f)]


def mk(i, name, size, state, cat, tags, prog, tracker):
    h = ("%040x" % (0xABC000 + i))
    TORRENTS[h] = {
        "name": name, "size": size, "total_size": size, "progress": prog, "state": state, "category": cat, "tags": tags,
        "dlspeed": 0, "upspeed": 0, "num_seeds": 3, "num_complete": 20, "num_leechs": 2, "num_incomplete": 8,
        "ratio": 0.5 * i, "eta": 8640000, "priority": i + 1, "added_on": 1700000000 + i * 3600,
        "completion_on": 1700100000 if prog >= 1 else -1, "tracker": tracker, "trackers_count": 1 if tracker else 0,
        "save_path": "/downloads/" + (cat or ""), "downloaded": int(size * prog), "uploaded": int(size * prog * 0.5),
        "amount_left": int(size * (1 - prog)), "private": False, "dl_limit": -1, "up_limit": 0,
        "magnet_uri": "magnet:?xt=urn:btih:" + h, "infohash_v1": h, "infohash_v2": "", "comment": "", "content_path": "/downloads/" + name,
        "availability": 1.5, "time_active": 4000 + i * 100, "seeding_time": 100, "has_metadata": True,
        "has_tracker_error": False, "has_tracker_warning": False, "has_other_announce_error": False, "reannounce": 600,
        "_hash": h, "added_by": "x", "auto_tmm": False, "seq_dl": False, "f_l_piece_prio": False, "force_start": False, "super_seeding": False,
    }


def init_synthetic(n):
    """n generated torrents for load testing; every 5th downloads so deltas flow each tick."""
    for i in range(n):
        state = ["downloading", "stalledDL", "uploading", "stoppedUP", "queuedDL"][i % 5]
        mk(i, "Synthetic torrent %06d with a moderately long name.mkv" % i, (1 + i % 997) * 1024 * 1024, state,
           ["", "tv", "tv/shows", "movies"][i % 4], "a,b" if i % 3 == 0 else "", round((i % 100) / 100, 3), "http://tracker%d.example/announce" % (i % 7))


def init():
    if ARGS.torrents:
        return init_synthetic(ARGS.torrents)
    names = ["ubuntu-24.04.iso", "debian-12-netinst.iso", "archlinux-2026.11.iso", "Big.Buck.Bunny.mkv", "Sintel.2010.mkv",
             "Some.Show.S01E01.mkv", "Some.Show.S01E02.mkv", "fedora-workstation.iso", "gentoo-minimal.iso", "linux-6.12.tar.xz",
             "public-domain-music.flac", "cc-photos.zip"]
    states = ["downloading", "downloading", "stalledDL", "uploading", "stalledUP", "downloading", "stoppedDL", "uploading",
              "queuedDL", "checkingUP", "stoppedUP", "error"]
    cats = ["", "", "", "movies", "movies", "tv/shows", "tv/shows", "", "", "", "", ""]
    tags = ["linux", "linux", "linux,hd", "hd", "", "", "", "linux", "", "", "", ""]
    for i, n in enumerate(names):
        prog = 1.0 if states[i] in ("uploading", "stalledUP", "stoppedUP", "checkingUP") else round(random.random() * 0.9, 3)
        mk(i, n, (1 + i) * 700 * 1024 * 1024, states[i], cats[i], tags[i], prog, "http://tracker%d.example:6969/announce" % (i % 3) if i != 10 else "")
    TORRENTS[list(TORRENTS)[-1]]["has_tracker_error"] = True


def tick():
    global RID
    while True:
        time.sleep(1.0)
        with STATE_LOCK:
            RID += 1
            ch = {h: dict(d) for h, d in PENDING.items()}
            PENDING.clear()
            if PENDING_REMOVED:
                REMOVED[RID] = list(PENDING_REMOVED)
                PENDING_REMOVED.clear()
            for h, t in TORRENTS.items():
                d = ch.get(h, {})
                if t["state"] == "downloading":
                    t["progress"] = min(1.0, round(t["progress"] + 0.004, 4))
                    sp = random.randint(200_000, 3_000_000)
                    d.update(progress=t["progress"], dlspeed=sp, eta=int(t["size"] * (1 - t["progress"]) / sp), downloaded=int(t["size"] * t["progress"]))
                    if t["progress"] >= 1.0:
                        t["state"] = "uploading"
                        d.update(state="uploading", completion_on=int(time.time()), dlspeed=0)
                        LOG.append({"id": len(LOG), "message": "%s finished downloading" % t["name"], "timestamp": int(time.time() * 1000), "type": 2})
                elif t["state"] == "uploading":
                    sp = random.randint(10_000, 500_000)
                    d.update(upspeed=sp)
                    t["upspeed"] = sp
                if d:
                    t.update(d)
                    ch[h] = d
            CHANGES[RID] = ch
            LOG.append({"id": len(LOG), "message": "tick %d" % RID, "timestamp": int(time.time() * 1000), "type": 1})
            for old in [r for r in CHANGES if r < RID - 50]:
                CHANGES.pop(old, None)
                REMOVED.pop(old, None)


def server_state():
    dl = sum(t["dlspeed"] for t in TORRENTS.values())
    ul = sum(t["upspeed"] for t in TORRENTS.values())
    return {"dl_info_speed": dl, "up_info_speed": ul, "dl_info_data": 123456789, "up_info_data": 98765432, "dl_rate_limit": 0, "up_rate_limit": 1048576,
            "dht_nodes": 321, "free_space_on_disk": 250 * 1024 ** 3, "connection_status": "connected", "use_alt_speed_limits": SERVER["alt"],
            "session_state": "paused" if SERVER["paused"] else "running", "queueing": True, "refresh_interval": 1000,
            "last_external_address_v4": "203.0.113.7", "alltime_dl": 5 * 1024 ** 4, "alltime_ul": 7 * 1024 ** 4, "global_ratio": "1.40",
            "read_cache_hits": "12.5", "total_buffers_size": 1048576, "write_cache_overload": "0", "read_cache_overload": "0",
            "queued_io_jobs": 0, "average_time_queue": 3, "total_queued_size": 0, "total_peer_connections": 42, "queued_tracker_announces": 0,
            "request_latency": 1, "total_wasted_session": 4096}


def public(t):
    return {k: v for k, v in t.items() if not k.startswith("_")}


def maindata(rid):
    with STATE_LOCK:
        base = {"rid": RID, "server_state": server_state()}
        if rid == 0 or rid < RID - 49:
            base.update(full_update=True, torrents={h: public(t) for h, t in TORRENTS.items()}, categories=CATEGORIES, tags=TAGS,
                        trackers={})
            trk = {}
            for h, t in TORRENTS.items():
                if t["tracker"]:
                    trk.setdefault(t["tracker"], []).append(h)
            base["trackers"] = trk
            return base
        merged, removed = {}, []
        for r in range(rid + 1, RID + 1):
            for h, d in CHANGES.get(r, {}).items():
                merged.setdefault(h, {}).update(d)
            removed += REMOVED.get(r, [])
        if merged:
            base["torrents"] = {h: d for h, d in merged.items() if h in TORRENTS}
        if removed:
            base["torrents_removed"] = removed
        return base


FILES = [{"index": 0, "name": "release/readme.txt", "size": 2048, "progress": 1.0, "priority": 1, "is_seed": False, "piece_range": [0, 0], "availability": 1.0},
         {"index": 1, "name": "release/data/main.bin", "size": 700 * 1024 * 1024, "progress": 0.5, "priority": 1, "is_seed": False, "piece_range": [1, 400], "availability": 1.2},
         {"index": 2, "name": "release/data/extra.bin", "size": 5 * 1024 * 1024, "progress": 0.0, "priority": 0, "is_seed": False, "piece_range": [401, 410], "availability": 0.8}]


class H(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *a):
        if ARGS.verbose:
            super().log_message(*a)

    def send(self, code=200, body=b"", ctype="text/plain", headers=None):
        if isinstance(body, (dict, list)):
            body = json.dumps(body).encode()
            ctype = "application/json"
        elif isinstance(body, str):
            body = body.encode()
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        for k, v in (headers or {}).items():
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(body)

    def authed(self):
        if ARGS.api_key and self.headers.get("Authorization") == "Bearer " + ARGS.api_key:
            return True
        c = self.headers.get("Cookie", "")
        return any(t in c for t in TOKENS)

    def handle_any(self, method):
        u = urlparse(self.path)
        q = {k: v[0] for k, v in parse_qs(u.query).items()}
        n = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(n) if n else b""
        if method == "POST" and "multipart" not in (self.headers.get("Content-Type") or ""):
            q.update({k: v[0] for k, v in parse_qs(raw.decode(errors="replace")).items()})
        parts = u.path.strip("/").split("/")
        if len(parts) != 4 or parts[:2] != ["api", "v2"]:
            return self.send(404, "Not Found")
        scope, action = parts[2], parts[3]
        if (scope, action) == ("auth", "login"):
            if q.get("username") == ARGS.user and q.get("password") == ARGS.password:
                tok = "t%d" % random.randint(1, 10 ** 9)
                TOKENS.add(tok)
                return self.send(200, "Ok.", headers={"Set-Cookie": "QBT_SID_%d=%s; HttpOnly; Path=/" % (ARGS.port, tok)})
            return self.send(401, "Fails.")
        if not self.authed():
            return self.send(403, "Forbidden")
        if (scope, action) in POST_ONLY and method != "POST":
            return self.send(405, "Method Not Allowed")
        r = ROUTES.get((scope, action))
        if r is None:
            return self.send(200, "") if method == "POST" else self.send(404, "Not Found")
        out = r(q)
        if isinstance(out, tuple):
            return self.send(*out)
        return self.send(200, out)

    def do_GET(self):
        self.handle_any("GET")

    def do_POST(self):
        self.handle_any("POST")


def stop_start(state_fn):
    def f(q):
        with STATE_LOCK:
            hs = list(TORRENTS) if q.get("hashes") == "all" else (q.get("hashes") or "").split("|")
            for h in hs:
                if h in TORRENTS:
                    state_fn(TORRENTS[h])
        return ""
    return f


def stop(t):
    t["state"] = "stoppedUP" if t["progress"] >= 1 else "stoppedDL"
    t["dlspeed"] = 0
    t["upspeed"] = 0
    PENDING.setdefault(t["_hash"], {}).update(state=t["state"], dlspeed=0, upspeed=0)


def start(t):
    t["state"] = "uploading" if t["progress"] >= 1 else "downloading"
    PENDING.setdefault(t["_hash"], {}).update(state=t["state"])


def delete(q):
    with STATE_LOCK:
        global RID
        for h in (q.get("hashes") or "").split("|"):
            if TORRENTS.pop(h, None) is not None:
                PENDING_REMOVED.append(h)
    return ""


def set_cat(q):
    with STATE_LOCK:
        for h in (q.get("hashes") or "").split("|"):
            if h in TORRENTS:
                TORRENTS[h]["category"] = q.get("category", "")
                PENDING.setdefault(h, {})["category"] = q.get("category", "")
    return ""


def peers(q):
    with STATE_LOCK:
        return {"rid": RID, "full_update": q.get("rid") == "0", "show_flags": True, "peers": {
            "10.0.0.%d:6881" % i: {"ip": "10.0.0.%d" % i, "port": 6881, "client": "qBittorrent/5.1", "connection": "BT", "country": "Germany", "country_code": "de",
                                   "dl_speed": 1000 * i, "up_speed": 500 * i, "downloaded": 10 ** 6 * i, "uploaded": 10 ** 5 * i, "progress": i / 10, "relevance": i / 10,
                                   "flags": "D K", "files": "readme.txt"} for i in range(1, 6)}}


def log_main(q):
    last = int(q.get("last_known_id", -1))
    return [e for e in LOG if e["id"] > last][-200:]


CLIENTDATA = {}


def clientdata_load(q):
    keys = json.loads(q.get("keys", "[]"))
    return {k: CLIENTDATA[k] for k in keys if k in CLIENTDATA}


def clientdata_store(q):
    CLIENTDATA.update(json.loads(q["data"]))
    return ""


ROUTES = {
    ("clientdata", "load"): clientdata_load, ("clientdata", "store"): clientdata_store,
    ("torrents", "downloadLimit"): lambda q: {h: -1 for h in (q.get("hashes") or "").split("|")},
    ("torrents", "uploadLimit"): lambda q: {h: 51200 for h in (q.get("hashes") or "").split("|")},
    ("transfer", "speedLimitsMode"): lambda q: "1" if SERVER["alt"] else "0",
    ("transfer", "setSpeedLimitsMode"): lambda q: SERVER.update(alt=q.get("mode") == "1") or "",
    ("app", "processInfo"): lambda q: {"launch_time": 1700000000},
    ("torrents", "pieceHashes"): lambda q: ["aa", "bb"],
    ("app", "networkInterfaceAddressList"): lambda q: ["10.0.0.5", "fe80::1"],

    ("app", "version"): lambda q: "v5.1.0", ("app", "webapiVersion"): lambda q: "2.17.0",
    ("app", "buildInfo"): lambda q: {"qt": "6.8.0", "libtorrent": "2.0.10", "boost": "1.86", "openssl": "3.3", "zlib": "1.3", "bitness": 64, "platform": "linux"},
    ("app", "preferences"): lambda q: PREFS,
    ("app", "setPreferences"): lambda q: (PREFS.update(json.loads(q["json"])) or ""),
    ("app", "defaultSavePath"): lambda q: "/downloads",
    ("app", "getFreeSpaceAtPath"): lambda q: str(250 * 1024 ** 3),
    ("app", "getDirectoryContent"): lambda q: [{"name": n, "type": "dir"} for n in ("downloads", "media", "tmp")] + ([{"name": "notes.txt", "type": "file", "size": 12}] if q.get("mode") == "all" else []),
    ("app", "cookies"): lambda q: [{"name": "uid", "domain": "tracker.example", "path": "/", "value": "123", "expirationDate": 0}],
    ("app", "rotateAPIKey"): lambda q: {"apiKey": "NEW-ROTATED-KEY"},
    ("app", "networkInterfaceList"): lambda q: [{"name": "Any interface", "value": ""}, {"name": "eth0", "value": "eth0"}],
    ("sync", "maindata"): lambda q: maindata(int(q.get("rid", 0))),
    ("sync", "torrentPeers"): peers,
    ("transfer", "info"): lambda q: server_state(),
    ("transfer", "getSpeedLimits"): lambda q: {"dl_limit": 0, "up_limit": 1048576, "alt_dl_limit": 524288, "alt_up_limit": 262144},
    ("transfer", "toggleSpeedLimitsMode"): lambda q: SERVER.update(alt=not SERVER["alt"]) or "",
    ("transfer", "pauseSession"): lambda q: SERVER.update(paused=True) or "",
    ("transfer", "resumeSession"): lambda q: SERVER.update(paused=False) or "",
    ("torrents", "start"): stop_start(start), ("torrents", "stop"): stop_start(stop), ("torrents", "delete"): delete,
    ("torrents", "setCategory"): set_cat,
    ("torrents", "categories"): lambda q: CATEGORIES,
    ("torrents", "tags"): lambda q: TAGS,
    ("torrents", "properties"): lambda q: (lambda t: {"save_path": t["save_path"], "total_size": t["size"], "pieces_num": 1000, "piece_size": t["size"] // 1000,
        "pieces_have": int(1000 * t["progress"]), "addition_date": t["added_on"], "completion_date": t["completion_on"], "created_by": "mock", "creation_date": 1690000000,
        "time_elapsed": t["time_active"], "seeding_time": 100, "eta": t["eta"], "nb_connections": 4, "nb_connections_limit": 100, "total_downloaded": t["downloaded"],
        "total_uploaded": t["uploaded"], "dl_speed": t["dlspeed"], "up_speed": t["upspeed"], "dl_speed_avg": 1000, "up_speed_avg": 500, "dl_limit": -1, "up_limit": -1,
        "seeds": 3, "seeds_total": 20, "peers": 2, "peers_total": 8, "share_ratio": t["ratio"], "reannounce": 300, "infohash_v1": t["infohash_v1"], "comment": "hello",
        "is_private": False, "last_seen": 1700000500})(TORRENTS[q["hash"]]) if q.get("hash") in TORRENTS else ("404", ),
    ("torrents", "trackers"): lambda q: [{"url": "http://tracker0.example:6969/announce", "status": 2, "tier": 0, "num_peers": 5, "num_seeds": 3, "num_leeches": 2, "num_downloaded": 100, "msg": "",
        "endpoints": [{"name": "http://198.51.100.1:6969/announce", "status": 2, "num_peers": 5, "num_seeds": 3, "num_leeches": 2, "num_downloaded": 100, "msg": ""}]},
        {"url": "** [DHT] **", "status": 2, "tier": -1, "num_peers": 40, "msg": ""}],
    ("torrents", "webseeds"): lambda q: [{"url": "http://mirror.example/file"}],
    ("torrents", "files"): lambda q: FILES,
    ("torrents", "pieceStates"): lambda q: [2] * 300 + [1] * 20 + [0] * 680,
    ("torrents", "pieceAvailability"): lambda q: [random.randint(1, 5) for _ in range(1000)],
    ("torrents", "info"): torrents_info,
    ("torrents", "count"): lambda q: str(len(TORRENTS)),
    ("log", "main"): log_main, ("log", "peers"): lambda q: [],
    ("search", "start"): lambda q: {"id": 1},
    ("search", "status"): lambda q: [{"id": 1, "status": "Stopped", "total": 3}],
    ("search", "results"): lambda q: {"status": "Stopped", "total": 3, "results": [
        {"fileName": "Ubuntu 24.04", "fileUrl": "magnet:?xt=urn:btih:aaaa", "fileSize": 6 * 1024 ** 3, "nbSeeders": 120, "nbLeechers": 4, "engineName": "mock", "siteUrl": "http://mock", "descrLink": "http://mock/1", "pubDate": 1700000000},
        {"fileName": "Debian 12", "fileUrl": "magnet:?xt=urn:btih:bbbb", "fileSize": 600 * 1024 ** 2, "nbSeeders": 80, "nbLeechers": 2, "engineName": "mock", "siteUrl": "http://mock", "descrLink": "http://mock/2", "pubDate": 1690000000},
        {"fileName": "Arch", "fileUrl": "magnet:?xt=urn:btih:cccc", "fileSize": 900 * 1024 ** 2, "nbSeeders": 5, "nbLeechers": 1, "engineName": "mock", "siteUrl": "http://mock", "descrLink": "http://mock/3", "pubDate": 1680000000}]},
    ("search", "plugins"): lambda q: [{"name": "mock", "fullName": "Mock Engine", "version": "1.0", "enabled": True, "url": "http://mock", "supportedCategories": [{"id": "all", "name": "All"}]}],
    ("rss", "items"): lambda q: {"News": {"Feed A": {"uid": "{1}", "url": "http://feed.example/a", "title": "Feed A", "articles": [
        {"id": "1", "title": "Ubuntu 24.04 released", "date": "Mon, 01 Jan 2026", "isRead": False, "description": "<p>New <b>release</b></p>", "link": "http://x/1", "torrentURL": "magnet:?xt=urn:btih:1111"},
        {"id": "2", "title": "Debian update", "date": "Tue, 02 Jan 2026", "isRead": True, "description": "Point release", "link": "http://x/2"}]}}},
    ("rss", "rules"): lambda q: {"My rule": {"enabled": True, "mustContain": "ubuntu", "affectedFeeds": ["http://feed.example/a"], "useRegex": False}},
    ("rss", "matchingArticles"): lambda q: {"Feed A": ["Ubuntu 24.04 released"]},
    ("torrentcreator", "status"): lambda q: [{"taskID": "t1", "sourcePath": "/downloads/x", "status": "Finished", "progress": 100, "errorMessage": "", "timeAdded": 1700000000}],
    ("torrentcreator", "addTask"): lambda q: {"taskID": "t2"},
}


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=8080)
    ap.add_argument("--user", default="admin")
    ap.add_argument("--password", default="adminadmin")
    ap.add_argument("--api-key", default="test-key")
    ap.add_argument("--verbose", action="store_true")
    ap.add_argument("--torrents", type=int, default=0, help="serve N synthetic torrents instead of the 12 sample ones")
    ARGS = ap.parse_args()
    init()
    load_methods()
    threading.Thread(target=tick, daemon=True).start()
    print("mock qBittorrent on http://127.0.0.1:%d (user %s, API key %s)" % (ARGS.port, ARGS.user, ARGS.api_key), flush=True)
    ThreadingHTTPServer(("127.0.0.1", ARGS.port), H).serve_forever()
