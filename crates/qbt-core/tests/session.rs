use std::time::Duration;

use qbt_api::{AuthMode, ClientConfig};
use qbt_core::{
    AuthKind, ConnectionState, Event, MemorySecrets, Profile, ProfileStore, SecretStore, Session,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

async fn next(rx: &mut tokio::sync::mpsc::UnboundedReceiver<Event>) -> Event {
    tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("timeout")
        .expect("closed")
}

async fn mock_basic(s: &MockServer) {
    for (p, body) in [
        ("/api/v2/app/webapiVersion", "2.17.0"),
        ("/api/v2/app/version", "v5.1.0"),
    ] {
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(s)
            .await;
    }
    Mock::given(method("GET"))
        .and(path("/api/v2/app/buildInfo"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"qt":"6"}"#))
        .mount(s)
        .await;
}

#[tokio::test]
async fn connects_then_streams_maindata_and_runs_commands() {
    let s = MockServer::start().await;
    mock_basic(&s).await;
    Mock::given(method("GET")).and(path("/api/v2/sync/maindata"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"rid":1,"full_update":true,"torrents":{"a":{"name":"A"}},"server_state":{"refresh_interval":500}}"#))
        .mount(&s).await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/stop"))
        .respond_with(ResponseTemplate::new(409).set_body_string("nope"))
        .mount(&s)
        .await;
    let cfg = ClientConfig::new(&s.uri(), AuthMode::ApiKey("k".into())).unwrap();
    let (h, mut rx) = Session::spawn(tokio::runtime::Handle::current(), cfg, 1.0).unwrap();

    assert!(matches!(
        next(&mut rx).await,
        Event::State(ConnectionState::Connecting)
    ));
    let Event::Connected(info) = next(&mut rx).await else {
        panic!()
    };
    assert!(info.capabilities.download_file);
    assert_eq!(info.version, "v5.1.0");
    assert!(matches!(
        next(&mut rx).await,
        Event::State(ConnectionState::Connected)
    ));
    let Event::MainData(d) = next(&mut rx).await else {
        panic!()
    };
    assert!(d.full_update && d.torrents.contains_key("a"));

    let id = h.run(|c| async move { c.torrents_stop(&qbt_api::Hashes::All).await });
    loop {
        if let Event::CommandFinished(r) = next(&mut rx).await {
            assert_eq!(r.id, id);
            assert!(!r.ok && r.message.contains("nope"));
            break;
        }
    }
    h.close().await;
}

#[tokio::test]
async fn bad_credentials_fail_without_retry() {
    let s = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/auth/login"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&s)
        .await;
    let cfg = ClientConfig::new(
        &s.uri(),
        AuthMode::Cookie {
            username: "a".into(),
            password: "b".into(),
        },
    )
    .unwrap();
    let (_h, mut rx) = Session::spawn(tokio::runtime::Handle::current(), cfg, 1.0).unwrap();
    assert!(matches!(
        next(&mut rx).await,
        Event::State(ConnectionState::Connecting)
    ));
    assert!(matches!(
        next(&mut rx).await,
        Event::State(ConnectionState::Failed(_))
    ));
    tokio::time::sleep(Duration::from_millis(300)).await; // `expect(1)` verifies no second login
}

#[test]
fn profiles_roundtrip_atomically_and_secrets_stay_out_of_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("sub/config.toml");
    let mut st = ProfileStore::load(&file).unwrap();
    let mut p = Profile::new("home", "https://nas.lan/qbt/");
    p.auth = AuthKind::ApiKey;
    st.upsert(p.clone());
    st.set_last_used("home");
    st.settings_mut().polling_multiplier = 2.0;
    st.save().unwrap();
    let secrets = MemorySecrets::default();
    secrets.set("home", "TOPSECRET").unwrap();

    let text = std::fs::read_to_string(&file).unwrap();
    assert!(!text.contains("TOPSECRET"));
    let back = ProfileStore::load(&file).unwrap();
    assert_eq!(back.get("home"), Some(&p));
    assert_eq!(back.last_used(), Some("home"));
    assert_eq!(back.settings().polling_multiplier, 2.0);
    assert!(!file.with_extension("toml.tmp").exists());
    let cfg = p.client_config(&secrets.get("home").unwrap()).unwrap();
    assert!(matches!(cfg.auth, AuthMode::ApiKey(_)));
}

#[tokio::test]
async fn torrent_actions_dispatch_to_the_right_endpoints() {
    use wiremock::matchers::body_string_contains;
    let s = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/setCategory"))
        .and(body_string_contains("hashes=a%7Cb"))
        .and(body_string_contains("category=tv"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&s)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v2/torrents/filePrio"))
        .and(body_string_contains("id=0%7C2"))
        .and(body_string_contains("priority=7"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&s)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v2/app/getFreeSpaceAtPath"))
        .respond_with(ResponseTemplate::new(200).set_body_string("42"))
        .expect(1)
        .mount(&s)
        .await;
    let c =
        qbt_api::Client::new(ClientConfig::new(&s.uri(), AuthMode::ApiKey("k".into())).unwrap())
            .unwrap();
    qbt_core::actions::torrent_action(&c, "setCategory", "a|b", "tv")
        .await
        .unwrap();
    qbt_core::actions::torrent_action(&c, "filePrio", "a", "0,2\u{1f}7")
        .await
        .unwrap();
    assert!(
        qbt_core::actions::torrent_action(&c, "rename", "a|b", "x")
            .await
            .is_err(),
        "needs one torrent"
    );
    assert!(
        qbt_core::actions::torrent_action(&c, "bogus", "a", "")
            .await
            .is_err()
    );
    let r = qbt_core::actions::call_json(&c, "app", "getFreeSpaceAtPath", r#"{"path":"/data"}"#)
        .await
        .unwrap();
    assert_eq!(r.text(), "42");
}

#[test]
fn every_listed_torrent_action_is_handled() {
    let src =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/actions.rs")).unwrap();
    for a in qbt_core::actions::TORRENT_ACTIONS {
        assert!(
            src.contains(&format!("\"{a}\" =>")),
            "{a} listed but not handled"
        );
    }
}
