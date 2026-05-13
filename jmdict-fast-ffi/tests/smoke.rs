//! End-to-end smoke test for the FFI facade, exercising the surface that
//! foreign-language bindings will see.

use std::sync::Arc;

use jmdict_fast_ffi::{Dict, Error, MatchMode, MatchType, QueryOptions, Xref};

fn load() -> Arc<Dict> {
    if let Ok(p) = std::env::var("JMDICT_DATA") {
        return Dict::load(p).expect("load failed");
    }
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist");
    Dict::load(dist.to_string_lossy().into_owned()).expect("load failed")
}

#[test]
fn dict_handle_is_arc_and_shareable() {
    let dict = load();
    let cloned = Arc::clone(&dict);
    assert_eq!(dict.entry_count(), cloned.entry_count());
}

#[test]
fn lookup_exact_works_through_facade() {
    let dict = load();
    let results = dict.lookup_exact("猫".to_string());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.kanji[0].text, "猫");
    assert_eq!(results[0].match_type, MatchType::Exact);
}

#[test]
fn lookup_partial_returns_owned_vec() {
    let dict = load();
    let results = dict.lookup_partial("たべ".to_string());
    assert!(!results.is_empty());
}

#[test]
fn lookup_with_options_carries_filters() {
    let dict = load();
    let options = QueryOptions {
        mode: MatchMode::Prefix,
        common_only: true,
        pos: vec!["v1".into()],
        limit: Some(5),
        ..QueryOptions::default()
    };
    let results = dict
        .lookup_with_options("たべ".to_string(), options)
        .expect("options query failed");
    assert!(results.len() <= 5);
    for r in &results {
        assert!(r.entry.kanji.iter().any(|k| k.common) || r.entry.kana.iter().any(|k| k.common));
        assert!(r
            .entry
            .sense
            .iter()
            .any(|s| s.part_of_speech.iter().any(|p| p.contains("v1"))));
    }
}

#[test]
fn lookup_batch_pairs_terms_with_results() {
    let dict = load();
    let batch = dict
        .lookup_batch(
            vec!["猫".into(), "犬".into()],
            QueryOptions::default(),
        )
        .expect("batch failed");
    assert_eq!(batch.len(), 2);
    assert_eq!(batch[0].term, "猫");
    assert_eq!(batch[1].term, "犬");
    for b in &batch {
        assert!(!b.results.is_empty(), "expected results for {}", b.term);
    }
}

#[test]
fn lookup_gloss_through_facade() {
    let dict = load();
    let results = dict.lookup_gloss("cat".to_string());
    assert!(!results.is_empty());
    for r in &results {
        assert_eq!(r.match_type, MatchType::Gloss);
    }
}

#[test]
fn lookup_by_id_through_facade() {
    let dict = load();
    let neko = &dict.lookup_exact("猫".to_string())[0].entry;
    let id = neko.id.clone();
    let found = dict.lookup_by_id(id.clone()).expect("missing");
    assert_eq!(found.entry.id, id);
}

#[test]
fn resolve_xref_through_facade() {
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
fn iter_entries_paginates() {
    let dict = load();
    let first_ten = dict.iter_entries(0, 10);
    assert_eq!(first_ten.len(), 10);

    let total = dict.entry_count();
    // Asking past the end clamps gracefully.
    let none = dict.iter_entries(total + 5, 10);
    assert!(none.is_empty());

    // start near the end clamps `count` too.
    let tail = dict.iter_entries(total - 3, 10);
    assert_eq!(tail.len(), 3);
}

#[test]
fn errors_collapse_io_to_string() {
    // Loading an absent path surfaces the FFI-friendly variant.
    match Dict::load("/this/path/does/not/exist".to_string()) {
        Err(Error::DataNotFound) | Err(Error::Io { .. }) => {}
        Err(other) => panic!("expected DataNotFound or Io, got {other:?}"),
        Ok(_) => panic!("expected an error loading a non-existent path"),
    }
}
