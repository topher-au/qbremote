use qbt_api::{AuthMode, Client, ClientConfig, Error, Hashes};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, header, method, path},
};

fn cookie_client(server: &MockServer) -> Client {
    let cfg = ClientConfig::new(
        &server.uri(),
        AuthMode::Cookie {
            username: "admin".into(),
            password: "pw".into(),
        },
    )
    .unwrap();
    Client::new(cfg).unwrap()
}

#[tokio::test]
async fn login_ok_and_get_uses_query() {
    let s = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/auth/login"))
        .and(body_string_contains("username=admin"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("Ok.")
                .insert_header("set-cookie", "QBT_SID_8080=abc; Path=/"),
        )
        .expect(1)
        .mount(&s)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v2/app/version"))
        .and(wiremock::matchers::header("cookie", "QBT_SID_8080=abc"))
        .respond_with(ResponseTemplate::new(200).set_body_string("v5.1.0\n"))
        .mount(&s)
        .await;
    let c = cookie_client(&s);
    c.login().await.unwrap();
    assert_eq!(c.app_version().await.unwrap(), "v5.1.0");
}

#[tokio::test]
async fn login_401_is_unauthorized_and_403_is_banned() {
    let s = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/auth/login"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&s)
        .await;
    assert!(matches!(
        cookie_client(&s).login().await,
        Err(Error::Unauthorized)
    ));

    let s = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/auth/login"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&s)
        .await;
    assert!(matches!(
        cookie_client(&s).login().await,
        Err(Error::Banned)
    ));
}

#[tokio::test]
async fn expired_session_relogs_in_once_and_retries() {
    let s = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/auth/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Ok."))
        .expect(1)
        .mount(&s)
        .await;
    // First call 403, then success.
    Mock::given(method("GET"))
        .and(path("/api/v2/app/version"))
        .respond_with(ResponseTemplate::new(403))
        .up_to_n_times(1)
        .mount(&s)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v2/app/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("v5.1.0"))
        .mount(&s)
        .await;
    assert_eq!(cookie_client(&s).app_version().await.unwrap(), "v5.1.0");
}

#[tokio::test]
async fn persistent_403_surfaces_after_one_retry() {
    let s = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/auth/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Ok."))
        .expect(1)
        .mount(&s)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v2/app/version"))
        .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
        .expect(2)
        .mount(&s)
        .await;
    assert!(matches!(
        cookie_client(&s).app_version().await,
        Err(Error::Forbidden(_))
    ));
}

#[tokio::test]
async fn api_key_sends_bearer_and_never_logs_in() {
    let s = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/app/webapiVersion"))
        .and(header("authorization", "Bearer k123"))
        .respond_with(ResponseTemplate::new(200).set_body_string("2.17.0"))
        .expect(1)
        .mount(&s)
        .await;
    let c =
        Client::new(ClientConfig::new(&s.uri(), AuthMode::ApiKey("k123".into())).unwrap()).unwrap();
    let caps = c.capabilities().await.unwrap();
    assert!(caps.download_file && caps.basic_auth);
    // No request was sent to auth/login.
    assert!(
        s.received_requests()
            .await
            .unwrap()
            .iter()
            .all(|r| !r.url.path().contains("auth/login"))
    );
}

#[tokio::test]
async fn post_actions_use_form_bodies_with_encoded_hashes() {
    let s = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/delete"))
        .and(body_string_contains("hashes=a%7Cb"))
        .and(body_string_contains("deleteFiles=true"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&s)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/stop"))
        .and(body_string_contains("hashes=all"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&s)
        .await;
    let c =
        Client::new(ClientConfig::new(&s.uri(), AuthMode::ApiKey("k".into())).unwrap()).unwrap();
    c.torrents_delete(&Hashes::Some(vec!["a".into(), "b".into()]), true)
        .await
        .unwrap();
    c.torrents_stop(&Hashes::All).await.unwrap();
}

#[tokio::test]
async fn sub_path_base_url_and_no_csrf_headers() {
    let s = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/qbt/api/v2/app/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("v5"))
        .mount(&s)
        .await;
    let c = Client::new(
        ClientConfig::new(&format!("{}/qbt", s.uri()), AuthMode::ApiKey("k".into())).unwrap(),
    )
    .unwrap();
    c.app_version().await.unwrap();
    let reqs = s.received_requests().await.unwrap();
    for h in ["origin", "referer", "sec-fetch-site"] {
        assert!(!reqs[0].headers.contains_key(h), "{h} must not be sent");
    }
}

#[tokio::test]
async fn status_mapping() {
    let s = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/topPrio"))
        .respond_with(
            ResponseTemplate::new(409).set_body_string("Torrent queueing must be enabled"),
        )
        .mount(&s)
        .await;
    let c =
        Client::new(ClientConfig::new(&s.uri(), AuthMode::ApiKey("k".into())).unwrap()).unwrap();
    let e = c
        .call("torrents", "topPrio", qbt_api::Params::new())
        .await
        .unwrap_err();
    assert!(matches!(&e, Error::Conflict(m) if m.contains("queueing")));
}

#[test]
fn forbidden_headers_rejected() {
    let mut cfg = ClientConfig::new("http://x", AuthMode::ApiKey("k".into())).unwrap();
    cfg.extra_headers.push(("Origin".into(), "http://x".into()));
    assert!(matches!(Client::new(cfg), Err(Error::Config(_))));
}

#[test]
fn secrets_are_redacted_in_debug() {
    let p: qbt_api::models::Preferences = serde_json::from_str(
        r#"{"web_ui_api_key":"SECRET","proxy_password":"SECRET","save_path":"/d"}"#,
    )
    .unwrap();
    let d = format!("{p:?} {:?}", AuthMode::ApiKey("SECRET".into()));
    assert!(!d.contains("SECRET") && d.contains("/d"));
}

/// Coverage gate (part 1): every row of the inventory tables is in the endpoint table.
#[test]
fn endpoint_table_matches_inventory() {
    let inv = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/api-inventory.md"
    ))
    .unwrap();
    let mut scope = String::new();
    let mut missing = vec![];
    let mut seen = 0;
    for line in inv.lines() {
        if let Some(s) = line.strip_prefix("### ") {
            scope = s.trim().to_owned();
            continue;
        }
        let Some(rest) = line.strip_prefix("| ") else {
            continue;
        };
        let cols: Vec<&str> = rest.split('|').collect();
        let first = cols[0].trim();
        if first == "Action" || first.starts_with("---") || scope.is_empty() {
            continue;
        }
        // Skip tables with no M column (log, torrentcreator are fine; method col is the 2nd for most).
        // "a / b" rows name two actions. Method is the second column when it is empty or `P`.
        let post = cols.get(1).map(|c| c.trim() == "P").unwrap_or(false);
        for name in first.split(" / ") {
            let name = name.trim().trim_matches('`');
            seen += 1;
            match qbt_api::endpoints::lookup(&scope, name) {
                Some(ep) if ep.post == post => {}
                _ => missing.push(format!("{scope}/{name} (post={post})")),
            }
        }
    }
    assert!(missing.is_empty(), "mismatched: {missing:?}");
    assert_eq!(
        seen,
        qbt_api::endpoints::ENDPOINTS.len(),
        "table size differs from inventory"
    );
}

#[tokio::test]
async fn download_streams_to_disk_and_cleans_up_on_error() {
    let s = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/torrents/downloadFile"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![7u8; 100_000]))
        .mount(&s)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v2/torrents/export"))
        .respond_with(ResponseTemplate::new(404).set_body_string("no such torrent"))
        .mount(&s)
        .await;
    let c =
        Client::new(ClientConfig::new(&s.uri(), AuthMode::ApiKey("k".into())).unwrap()).unwrap();
    let dir = std::env::temp_dir().join(format!("qbt-dl-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("out.bin");
    let n = c
        .download_to(
            "torrents",
            "downloadFile",
            qbt_api::Params::new().str("hash", "h"),
            &f,
        )
        .await
        .unwrap();
    assert_eq!(n, 100_000);
    assert_eq!(std::fs::metadata(&f).unwrap().len(), 100_000);
    let g = dir.join("missing.torrent");
    let e = c
        .download_to("torrents", "export", qbt_api::Params::new(), &g)
        .await
        .unwrap_err();
    assert!(matches!(e, Error::NotFound(_)));
    assert!(!g.exists());
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn capability_table_is_consistent_with_endpoints_and_versions() {
    use qbt_api::{
        Capabilities, WebApiVersion,
        capabilities::{MIN_SUPPORTED, MIN_VERSIONS},
    };
    for (scope, action, _) in MIN_VERSIONS {
        assert!(
            qbt_api::endpoints::lookup(scope, action).is_some(),
            "{scope}/{action} not in endpoint table"
        );
    }
    let old = Capabilities::from_version(WebApiVersion::new(2, 11, 2));
    assert!(old.supported && !old.download_file && !old.session_pause && !old.speed_limits);
    assert!(
        old.supports("torrents", "start"),
        "undocumented-minimum endpoints are assumed present"
    );
    let new = Capabilities::from_version("2.17.0".parse().unwrap());
    assert!(
        new.download_file
            && new.session_pause
            && new.client_data
            && new.supports("torrents", "cancelFetchMetadata")
    );
    assert!(
        !Capabilities::from_version(WebApiVersion::new(2, 10, 4)).supported,
        "4.x/early-5.0 betas are unsupported"
    );
    assert_eq!(MIN_SUPPORTED, WebApiVersion::new(2, 11, 0));
    assert_eq!(
        Capabilities::requirement("transfer", "getSpeedLimits").unwrap(),
        "requires WebAPI ≥ 2.16.0"
    );
}

#[tokio::test]
async fn empty_404_marks_an_action_unsupported_but_a_message_does_not() {
    let s = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/torrents/count"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&s)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v2/torrents/properties"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Torrent hash was not found"))
        .mount(&s)
        .await;
    let c =
        Client::new(ClientConfig::new(&s.uri(), AuthMode::ApiKey("k".into())).unwrap()).unwrap();
    assert!(c.torrents_count().await.is_err());
    assert!(c.is_unsupported("torrents", "count"));
    assert!(c.torrents_properties("x").await.is_err());
    assert!(!c.is_unsupported("torrents", "properties"));
}

#[tokio::test]
async fn clientdata_roundtrip_encoding() {
    let s = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/clientdata/load"))
        .and(wiremock::matchers::query_param(
            "keys",
            r#"["qbremote.ui"]"#,
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"qbremote.ui":{"transferColumns":"name"}}"#),
        )
        .expect(1)
        .mount(&s)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v2/clientdata/store"))
        .and(body_string_contains("data=%7B%22qbremote.ui%22"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&s)
        .await;
    let c =
        Client::new(ClientConfig::new(&s.uri(), AuthMode::ApiKey("k".into())).unwrap()).unwrap();
    let v = c
        .clientdata_load(&["qbremote.ui".to_owned()])
        .await
        .unwrap();
    assert_eq!(v["qbremote.ui"]["transferColumns"], "name");
    let mut m = serde_json::Map::new();
    m.insert(
        "qbremote.ui".into(),
        serde_json::json!({"transferColumns": "name"}),
    );
    c.clientdata_store(&m).await.unwrap();
}
