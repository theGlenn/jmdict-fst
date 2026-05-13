//! Rust-side smoke test for the Dart-facing API. These exercise the exact
//! types and methods that `flutter_rust_bridge_codegen` will export — a
//! green run means the surface is internally consistent before we generate
//! Dart bindings.

use jmdict_fast_flutter::api::{Dict, MatchMode, MatchType, QueryOptions, Xref};

fn load() -> Dict {
    if let Ok(p) = std::env::var("JMDICT_DATA") {
        return Dict::load(p).expect("load failed");
    }
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist");
    Dict::load(dist.to_string_lossy().into_owned()).expect("load failed")
}

#[test]
fn lookup_exact_through_frb_surface() {
    let dict = load();
    let results = dict.lookup_exact("猫".to_string());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.kanji[0].text, "猫");
    assert!(matches!(results[0].match_type, MatchType::Exact));
}

#[test]
fn lookup_partial_through_frb_surface() {
    let dict = load();
    let results = dict.lookup_partial("たべ".to_string());
    assert!(!results.is_empty());
}

#[test]
fn lookup_gloss_through_frb_surface() {
    let dict = load();
    let results = dict.lookup_gloss("cat".to_string());
    assert!(results.iter().any(|r| matches!(r.match_type, MatchType::Gloss)));
}

#[test]
fn lookup_with_options_through_frb_surface() {
    let dict = load();
    let options = QueryOptions {
        mode: MatchMode::Prefix,
        common_only: true,
        pos: vec!["v1".into()],
        misc: Vec::new(),
        field: Vec::new(),
        dialect: Vec::new(),
        limit: Some(5),
        max_distance: 2,
    };
    let results = dict
        .lookup_with_options("たべ".to_string(), options)
        .expect("options query failed");
    assert!(results.len() <= 5);
}

#[test]
fn lookup_batch_through_frb_surface() {
    let dict = load();
    let options = QueryOptions {
        mode: MatchMode::Exact,
        common_only: false,
        pos: Vec::new(),
        misc: Vec::new(),
        field: Vec::new(),
        dialect: Vec::new(),
        limit: None,
        max_distance: 2,
    };
    let batch = dict
        .lookup_batch(vec!["猫".into(), "犬".into()], options)
        .expect("batch failed");
    assert_eq!(batch.len(), 2);
    assert_eq!(batch[0].term, "猫");
    assert!(!batch[0].results.is_empty());
}

#[test]
fn lookup_by_id_through_frb_surface() {
    let dict = load();
    let neko = &dict.lookup_exact("猫".to_string())[0].entry;
    let id = neko.id.clone();
    let found = dict.lookup_by_id(id.clone()).expect("missing");
    assert_eq!(found.entry.id, id);
}

#[test]
fn resolve_xref_through_frb_surface() {
    let dict = load();
    let xref = Xref {
        term: "猫".to_string(),
        reading: None,
        sense_index: None,
    };
    let results = dict.resolve_xref(xref);
    assert!(results.iter().any(|r| r.entry.kanji[0].text == "猫"));
}

#[test]
fn iter_entries_paginates_through_frb_surface() {
    let dict = load();
    let ten = dict.iter_entries(0, 10);
    assert_eq!(ten.len(), 10);
    let total = dict.entry_count();
    let tail = dict.iter_entries(total - 3, 10);
    assert_eq!(tail.len(), 3);
}

#[test]
fn dict_is_send_and_sync() {
    // flutter_rust_bridge ships Dict to other Dart isolates via send-ports,
    // so it must be Send + Sync. Lock the invariant into the type system.
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Dict>();
}
