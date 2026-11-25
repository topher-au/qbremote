//! Coverage gate: every action in the endpoint table has a typed client method.
use qbt_api::endpoints::ENDPOINTS;

#[test]
fn every_endpoint_has_typed_method() {
    let src = ["api.rs", "client.rs"]
        .iter()
        .map(|f| {
            std::fs::read_to_string(format!("{}/src/{f}", env!("CARGO_MANIFEST_DIR"))).unwrap()
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let missing: Vec<_> = ENDPOINTS
        .iter()
        .filter(|e| !src.contains(&format!("\"{}\"", e.action)))
        .map(|e| format!("{}/{}", e.scope, e.action))
        .collect();
    assert!(missing.is_empty(), "no typed method for {missing:?}");
    // Where an action name repeats across scopes, require the scope-qualified call.
    for e in ENDPOINTS {
        let dup = ENDPOINTS.iter().filter(|o| o.action == e.action).count() > 1;
        if !dup {
            continue;
        }
        let call = format!("\"{}\", \"{}\"", e.scope, e.action);
        let macro_form = format!("=> \"{}\"", e.action);
        assert!(
            src.contains(&call) || (e.scope == "torrents" && src.contains(&macro_form)),
            "{}/{} not wired",
            e.scope,
            e.action
        );
    }
}
