//! Integration tests that exercise the full library against real JMdict data.
//!
//! These need `dist/` to exist (run `cargo xtask generate` or set
//! `JMDICT_DATA`). They are intentionally separated from unit tests so the
//! `cargo test --lib` suite stays runnable without dictionary data.

use jmdict_fast::*;

fn create_test_dict() -> Dict {
    // `cargo test --test lookup` runs with CWD set to the crate, so the
    // workspace `dist/` is one directory up. Prefer the `JMDICT_DATA` env
    // var when set (CI may point it elsewhere); fall back to the workspace
    // sibling directory.
    if let Ok(p) = std::env::var("JMDICT_DATA") {
        return Dict::load(p).expect("load failed");
    }
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist");
    Dict::load(&dist).expect("load failed")
}

// --- lookup_exact -----------------------------------------------------------

#[test]
fn test_lookup_exact() {
    let dict = create_test_dict();

    let results = dict.lookup_exact("猫");

    assert!(!results.is_empty(), "Expected to find entries for 猫");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].match_type, MatchType::Exact);
    assert_eq!(results[0].match_key, "猫");
    assert_eq!(results[0].score, 1.0);
    assert!(results[0].deinflection.is_none());
    assert_eq!(results[0].entry.kana[0].text, "ねこ");
    assert_eq!(results[0].entry.kanji[0].text, "猫");
    assert_eq!(
        results[0].entry.sense[0].gloss[0].text,
        "cat (esp. the domestic cat, Felis catus)"
    );
}

#[test]
fn test_lookup_exact_verb() {
    let dict = create_test_dict();

    let results = dict.lookup_exact("食べる");

    assert!(!results.is_empty(), "Expected to find entries for 食べる");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.kana[0].text, "たべる");
    assert_eq!(results[0].entry.kanji[0].text, "食べる");
    assert_eq!(results[0].entry.sense[0].gloss[0].text, "to eat");
}

#[test]
fn test_lookup_exact_verb_no_deinflection_should_not_find() {
    let dict = create_test_dict();

    let results = dict.lookup_exact("食べます");

    assert!(
        results.is_empty(),
        "Expected to not find entries for 食べます"
    );
    assert_eq!(results.len(), 0);
}

#[test]
fn test_lookup_exact_multiple() {
    let dict = create_test_dict();

    let test_words = ["猫", "犬", "魚", "鳥", "花"];

    for word in test_words {
        let results = dict.lookup_exact(word);
        assert!(!results.is_empty(), "Expected to find entries for {}", word);
    }
}

// --- lookup_exact_with_deinflection -----------------------------------------

#[test]
fn test_lookup_exact_with_deinflection() {
    let dict = create_test_dict();
    let results = dict.lookup_exact_with_deinflection("たべます");
    assert!(!results.is_empty(), "Expected to find entries for たべます");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].match_type, MatchType::Deinflected);
    assert_eq!(results[0].score, 0.75);
    assert!(results[0].deinflection.is_some());
    let deinf = results[0].deinflection.as_ref().unwrap();
    assert_eq!(deinf.original_form, "たべます");
    assert_eq!(deinf.base_form, "たべる");
    assert!(!deinf.rules.is_empty());
    assert_eq!(results[0].entry.kana[0].text, "たべる");
    assert_eq!(results[0].entry.kanji[0].text, "食べる");

    let results = dict.lookup_exact_with_deinflection("食べます");
    assert!(!results.is_empty(), "Expected to find entries for 食べます");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entry.kana[0].text, "たべる");
    assert_eq!(results[0].entry.kanji[0].text, "食べる");
}

#[test]
fn test_lookup_exact_with_deinflection_adj() {
    let dict = create_test_dict();
    let results = dict.lookup_exact_with_deinflection("美しい");
    assert!(!results.is_empty(), "Expected to find entries for 美しい");
    assert_eq!(results.len(), 1);
    // Direct match should be Exact, not Deinflected
    assert_eq!(results[0].match_type, MatchType::Exact);
    assert_eq!(results[0].entry.kana[0].text, "うつくしい");
    assert_eq!(results[0].entry.kanji[0].text, "美しい");
}

#[test]
fn test_lookup_exact_with_deinflection_adj_should_not_find() {
    let dict = create_test_dict();

    let results = dict.lookup_exact_with_deinflection("美しいです");
    assert!(
        results.is_empty(),
        "Expected to not find entries for 美しいです"
    );
    assert_eq!(results.len(), 0);
}

#[test]
fn test_lookup_exact_with_deinflection_kanji() {
    let dict = create_test_dict();
    let results = dict.lookup_exact_with_deinflection("生");
    assert!(!results.is_empty(), "Expected to find entries for 生");
    assert_eq!(results.len(), 1);
}

// --- lookup_partial ---------------------------------------------------------

#[test]
fn test_lookup_partial() {
    let dict = create_test_dict();
    let results = dict.lookup_partial("たべ");
    assert!(!results.is_empty(), "Expected to find entries for たべ");

    assert!(
        results.len() > 3,
        "Expected to find more than 3 results for たべ"
    );
    assert!(
        results.iter().any(|r| r.entry.kana[0].text == "たべる"),
        "Expected to find たべる in results"
    );
    assert!(
        results.iter().any(|r| r.entry.kanji[0].text == "食べる"),
        "Expected to find 食べる in results"
    );
    assert!(
        results
            .iter()
            .any(|r| r.entry.sense[0].gloss[0].text == "to eat"),
        "Expected to find to eat in results"
    );
    assert!(
        results.iter().any(|r| r.match_type == MatchType::Prefix),
        "Expected some Prefix match types"
    );
}

#[test]
fn test_lookup_partial_in_depth() {
    let dict = create_test_dict();

    let partial_results = dict.lookup_partial("ね");
    assert!(
        !partial_results.is_empty(),
        "Partial lookup should find entries starting with ね"
    );

    let has_nekko = partial_results
        .iter()
        .any(|lr| lr.entry.kana.iter().any(|k| k.text.starts_with("ね")));
    assert!(
        has_nekko,
        "Partial results should include entries starting with ね"
    );

    let exact_neko = dict.lookup_exact("ねこ");
    let partial_neko = dict.lookup_partial("ねこ");
    assert!(
        partial_neko.len() >= exact_neko.len(),
        "Partial lookup should find at least as many results as exact lookup"
    );

    for window in partial_neko.windows(2) {
        assert!(
            window[0].score >= window[1].score,
            "Results should be sorted by score descending"
        );
    }
}

#[test]
fn test_prefix_results_dedup_by_id_and_keep_best_score() {
    let dict = create_test_dict();
    // `たべる` is itself an exact key; under prefix mode it should surface as
    // an Exact (score 1.0) match for its entry rather than being demoted to a
    // generic Prefix score because some other key for the same id was visited
    // first during FST traversal.
    let results = dict
        .lookup("たべる")
        .mode(MatchMode::Prefix)
        .execute()
        .unwrap();

    let mut ids: Vec<u64> = results.iter().map(|r| r.entry.id.parse().unwrap_or(0)).collect();
    let pre_dedup_len = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(pre_dedup_len, ids.len(), "results must be deduplicated by entry id");

    let taberu = results
        .iter()
        .find(|r| r.entry.kana.iter().any(|k| k.text == "たべる"))
        .expect("expected to find an entry for たべる");
    assert_eq!(taberu.match_type, MatchType::Exact);
    assert_eq!(taberu.score, 1.0);
}

// --- QueryBuilder -----------------------------------------------------------

#[test]
fn test_query_builder_exact() {
    let dict = create_test_dict();
    let results = dict.lookup("猫").execute().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].match_type, MatchType::Exact);
    assert_eq!(results[0].entry.kanji[0].text, "猫");
}

#[test]
fn test_query_builder_prefix() {
    let dict = create_test_dict();
    let results = dict.lookup("たべ").mode(MatchMode::Prefix).execute().unwrap();
    assert!(!results.is_empty());
    assert!(results.iter().any(|r| r.match_type == MatchType::Prefix));
    assert!(results.iter().any(|r| r.entry.kana[0].text == "たべる"));
}

#[test]
fn test_query_builder_deinflect() {
    let dict = create_test_dict();
    let results = dict.lookup("たべます").mode(MatchMode::Deinflect).execute().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].match_type, MatchType::Deinflected);
    assert!(results[0].deinflection.is_some());
    assert_eq!(results[0].entry.kana[0].text, "たべる");
}

#[test]
fn test_query_builder_default_mode_is_exact() {
    let dict = create_test_dict();
    let builder_results = dict.lookup("猫").execute().unwrap();
    let direct_results = dict.lookup_exact("猫");
    assert_eq!(builder_results.len(), direct_results.len());
    assert_eq!(builder_results[0].entry.id, direct_results[0].entry.id);
}

#[test]
fn test_query_builder_common_only() {
    let dict = create_test_dict();
    let results = dict.lookup("猫").common_only(true).execute().unwrap();
    assert_eq!(results.len(), 1);
    assert!(
        results[0].entry.kanji.iter().any(|k| k.common)
            || results[0].entry.kana.iter().any(|k| k.common)
    );
}

#[test]
fn test_query_builder_common_only_prefix() {
    let dict = create_test_dict();
    let all_results = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .execute()
        .unwrap();
    let common_results = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .common_only(true)
        .execute()
        .unwrap();
    assert!(common_results.len() <= all_results.len());
    for r in &common_results {
        assert!(
            r.entry.kanji.iter().any(|k| k.common)
                || r.entry.kana.iter().any(|k| k.common),
            "Expected all common_only results to have a common reading"
        );
    }
}

#[test]
fn test_query_builder_pos_filter() {
    let dict = create_test_dict();
    let results = dict.lookup("食べる").pos(&["v1"]).execute().unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0]
        .entry
        .sense
        .iter()
        .any(|s| s.part_of_speech.iter().any(|p| p.contains("v1"))));
}

#[test]
fn test_query_builder_pos_filter_excludes() {
    let dict = create_test_dict();
    // 猫 is a noun, not a verb — filtering for "v1" should exclude it
    let results = dict.lookup("猫").pos(&["v1"]).execute().unwrap();
    assert!(results.is_empty(), "猫 should not match v1 POS filter");
}

#[test]
fn test_query_builder_limit() {
    let dict = create_test_dict();
    let all_results = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .execute()
        .unwrap();
    assert!(all_results.len() > 3, "Need enough results to test limit");

    let limited = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .limit(2)
        .execute()
        .unwrap();
    assert_eq!(limited.len(), 2);
}

#[test]
fn test_query_builder_chained_filters() {
    let dict = create_test_dict();
    let results = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .common_only(true)
        .pos(&["v1"])
        .limit(10)
        .execute()
        .unwrap();
    assert!(!results.is_empty());
    assert!(results.len() <= 10);
    for r in &results {
        assert!(
            r.entry.kanji.iter().any(|k| k.common)
                || r.entry.kana.iter().any(|k| k.common)
        );
        assert!(r
            .entry
            .sense
            .iter()
            .any(|s| s.part_of_speech.iter().any(|p| p.contains("v1"))));
    }
}

// --- Fuzzy ------------------------------------------------------------------

#[test]
fn test_fuzzy_search_romaji() {
    let dict = create_test_dict();
    // "neko" is exact, "nko" is 1 edit away (missing 'e')
    let results = dict
        .lookup("nko")
        .mode(MatchMode::Fuzzy)
        .max_distance(1)
        .execute()
        .unwrap();
    assert!(
        results.iter().any(|r| r.entry.kana.iter().any(|k| k.text == "ねこ")),
        "Fuzzy search for 'nko' should find ねこ (neko)"
    );
    for r in &results {
        if r.match_type == MatchType::Fuzzy {
            assert!(r.score < 1.0, "Fuzzy results should have score < 1.0");
        }
    }
}

#[test]
fn test_fuzzy_search_exact_match_included() {
    let dict = create_test_dict();
    let results = dict
        .lookup("neko")
        .mode(MatchMode::Fuzzy)
        .max_distance(1)
        .execute()
        .unwrap();
    assert!(
        results.iter().any(|r| r.match_type == MatchType::Exact && r.match_key == "neko"),
        "Fuzzy search for exact term should include exact match"
    );
}

#[test]
fn test_fuzzy_search_max_distance() {
    let dict = create_test_dict();
    let results_d0 = dict
        .lookup("neko")
        .mode(MatchMode::Fuzzy)
        .max_distance(0)
        .execute()
        .unwrap();
    for r in &results_d0 {
        assert_eq!(r.match_type, MatchType::Exact, "Distance 0 should only return exact matches");
    }

    let results_d1 = dict
        .lookup("neko")
        .mode(MatchMode::Fuzzy)
        .max_distance(1)
        .execute()
        .unwrap();
    let results_d2 = dict
        .lookup("neko")
        .mode(MatchMode::Fuzzy)
        .max_distance(2)
        .execute()
        .unwrap();
    assert!(
        results_d2.len() >= results_d1.len(),
        "Higher distance should return at least as many results"
    );
}

#[test]
fn test_fuzzy_search_with_filters() {
    let dict = create_test_dict();
    let results = dict
        .lookup("neko")
        .mode(MatchMode::Fuzzy)
        .max_distance(2)
        .common_only(true)
        .limit(5)
        .execute()
        .unwrap();
    assert!(results.len() <= 5);
    for r in &results {
        assert!(
            r.entry.kanji.iter().any(|k| k.common)
                || r.entry.kana.iter().any(|k| k.common),
            "common_only filter should apply to fuzzy results"
        );
    }
}

#[test]
fn test_max_distance_is_clamped() {
    let dict = create_test_dict();
    // Asking for an absurd edit distance must not blow up the Levenshtein DFA;
    // QueryBuilder clamps to MAX_FUZZY_DISTANCE before constructing it.
    let results = dict
        .lookup("ねこ")
        .mode(MatchMode::Fuzzy)
        .max_distance(100)
        .limit(1)
        .execute()
        .expect("clamped fuzzy query should succeed");
    assert!(!results.is_empty());
}

// --- BatchQueryBuilder ------------------------------------------------------

#[test]
fn test_batch_lookup_basic() {
    let dict = create_test_dict();
    let results = dict
        .lookup_batch(&["猫", "犬", "食べる"])
        .execute()
        .unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, "猫");
    assert_eq!(results[1].0, "犬");
    assert_eq!(results[2].0, "食べる");
    for (term, entries) in &results {
        assert!(!entries.is_empty(), "Expected results for {}", term);
    }
}

#[test]
fn test_batch_lookup_with_filters() {
    let dict = create_test_dict();
    let results = dict
        .lookup_batch(&["猫", "食べる"])
        .common_only(true)
        .pos(&["n"])
        .execute()
        .unwrap();
    assert_eq!(results.len(), 2);
    assert!(!results[0].1.is_empty(), "猫 should match noun filter");
    assert!(results[1].1.is_empty(), "食べる should not match noun filter");
}

#[test]
fn test_batch_lookup_with_mode() {
    let dict = create_test_dict();
    let results = dict
        .lookup_batch(&["たべ"])
        .mode(MatchMode::Prefix)
        .limit(3)
        .execute()
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "たべ");
    assert!(results[0].1.len() <= 3);
    assert!(!results[0].1.is_empty());
}

#[test]
fn test_batch_lookup_empty_terms() {
    let dict = create_test_dict();
    let results = dict.lookup_batch(&[]).execute().unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_batch_lookup_matches_individual() {
    let dict = create_test_dict();
    let terms = &["猫", "犬"];
    let batch = dict.lookup_batch(terms).execute().unwrap();

    for (term, batch_entries) in &batch {
        let individual = dict.lookup(term).execute().unwrap();
        assert_eq!(
            batch_entries.len(),
            individual.len(),
            "Batch and individual results should match for {}",
            term
        );
    }
}

// --- LookupResultIter -------------------------------------------------------

#[test]
fn test_execute_iter_returns_same_as_execute() {
    let dict = create_test_dict();
    let collected: Vec<_> = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .execute_iter()
        .unwrap()
        .collect();
    let executed = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .execute()
        .unwrap();
    assert_eq!(collected.len(), executed.len());
    for (a, b) in collected.iter().zip(executed.iter()) {
        assert_eq!(a.entry.id, b.entry.id);
        assert_eq!(a.match_type, b.match_type);
        assert_eq!(a.score, b.score);
    }
}

#[test]
fn test_execute_iter_lazy_with_limit() {
    let dict = create_test_dict();
    let iter = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .limit(2)
        .execute_iter()
        .unwrap();
    let results: Vec<_> = iter.collect();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_execute_iter_with_filters() {
    let dict = create_test_dict();
    let results: Vec<_> = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .common_only(true)
        .pos(&["v1"])
        .limit(5)
        .execute_iter()
        .unwrap()
        .collect();
    assert!(!results.is_empty());
    assert!(results.len() <= 5);
    for r in &results {
        assert!(
            r.entry.kanji.iter().any(|k| k.common)
                || r.entry.kana.iter().any(|k| k.common)
        );
        assert!(r.entry.sense.iter().any(|s| {
            s.part_of_speech.iter().any(|p| p.contains("v1"))
        }));
    }
}

#[test]
fn test_execute_iter_partial_consumption() {
    let dict = create_test_dict();
    let mut iter = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .execute_iter()
        .unwrap();
    let first = iter.next();
    assert!(first.is_some(), "Should have at least one result");
    let second = iter.next();
    assert!(second.is_some(), "Should have more than one result for prefix たべ");
}

// --- Metadata, browsing, by-id ---------------------------------------------

#[test]
fn test_entry_count() {
    let dict = create_test_dict();
    let count = dict.entry_count();
    assert!(count > 10_000, "Expected more than 10,000 entries, got {}", count);
}

#[test]
fn test_version() {
    let dict = create_test_dict();
    let version = dict.version();
    assert_eq!(version.format_version, FORMAT_VERSION);
    assert!(!version.jmdict_version.is_empty(), "jmdict_version should not be empty");
    assert!(!version.generated_at.is_empty(), "generated_at should not be empty");
}

#[test]
fn test_lookup_by_id_roundtrip() {
    let dict = create_test_dict();
    let neko = &dict.lookup_exact("猫")[0].entry;
    let jmdict_id = neko.id.clone();

    let found = dict.lookup_by_id(&jmdict_id).expect("lookup_by_id failed");
    assert_eq!(found.entry.id, jmdict_id);
    assert_eq!(found.entry.kanji[0].text, "猫");
    assert_eq!(found.match_type, MatchType::Exact);
    assert_eq!(found.score, 1.0);
}

#[test]
fn test_lookup_by_id_missing() {
    let dict = create_test_dict();
    assert!(dict.lookup_by_id("0000000").is_none());
}

#[test]
fn test_get_by_seq_id() {
    let dict = create_test_dict();
    let entry = dict.get(0).expect("first entry should exist");
    assert!(!entry.id.is_empty());

    let n = dict.entry_count() as u64;
    assert!(dict.get(n).is_none());
    assert!(dict.get(n + 1_000_000).is_none());
}

#[test]
fn test_iter_entries_walks_every_entry() {
    let dict = create_test_dict();
    let count = dict.iter_entries().count();
    assert_eq!(count, dict.entry_count());
}

#[test]
fn test_entry_accessors_against_real_data() {
    let dict = create_test_dict();
    let neko = dict.lookup_exact("猫")[0].entry.clone();

    assert_eq!(neko.primary_kanji(), Some("猫"));
    assert_eq!(neko.primary_kana(), Some("ねこ"));
    assert_eq!(neko.headword(), Some("猫"));
    assert!(neko.is_common());

    let eng: Vec<&str> = neko.glosses("eng").collect();
    assert!(eng.iter().any(|g| g.contains("cat")));

    let pos = neko.parts_of_speech();
    assert!(pos.iter().any(|p| p.contains("n")));
}

// --- Sense filters (misc/field/dialect) ------------------------------------

#[test]
fn test_query_builder_misc_filter_uk() {
    let dict = create_test_dict();
    // "uk" = usually written in kana. Use a narrow prefix so the unfiltered
    // result count fits below the limit, otherwise both queries hit the
    // ceiling and the reduction assertion is meaningless.
    let all = dict
        .lookup("あいう")
        .mode(MatchMode::Prefix)
        .execute()
        .unwrap();
    let uk_only = dict
        .lookup("あいう")
        .mode(MatchMode::Prefix)
        .misc(&["uk"])
        .execute()
        .unwrap();
    assert!(uk_only.len() <= all.len(), "uk filter must not add results");
    for r in &uk_only {
        assert!(
            r.entry.sense.iter().any(|s| s.misc.iter().any(|m| m == "uk")),
            "entry {} kept under uk filter but no sense has misc=uk",
            r.entry.id
        );
    }
}

#[test]
fn test_query_builder_field_filter() {
    let dict = create_test_dict();
    // A prefix sweep over hiragana "い" with field=med should yield only
    // medicine-tagged entries.
    let med = dict
        .lookup("い")
        .mode(MatchMode::Prefix)
        .field(&["med"])
        .limit(50)
        .execute()
        .unwrap();
    for r in &med {
        assert!(
            r.entry.sense.iter().any(|s| s.field.iter().any(|f| f == "med")),
            "entry {} kept under field=med but no sense has field=med",
            r.entry.id
        );
    }
}

#[test]
fn test_query_builder_dialect_filter() {
    let dict = create_test_dict();
    // Kansai-ben filter — sparse but should yield only ksb-tagged entries.
    let ksb = dict
        .lookup("や")
        .mode(MatchMode::Prefix)
        .dialect(&["ksb"])
        .limit(20)
        .execute()
        .unwrap();
    for r in &ksb {
        assert!(
            r.entry.sense.iter().any(|s| s.dialect.iter().any(|d| d == "ksb")),
            "entry {} kept under dialect=ksb but no sense has dialect=ksb",
            r.entry.id
        );
    }
}

#[test]
fn test_query_builder_combined_pos_and_misc_per_sense_conjunction() {
    let dict = create_test_dict();
    // Verbs that are usually written in kana — every result must have a
    // SINGLE sense that is both a verb AND tagged "uk".
    let results = dict
        .lookup("い")
        .mode(MatchMode::Prefix)
        .pos(&["v"])
        .misc(&["uk"])
        .limit(30)
        .execute()
        .unwrap();
    for r in &results {
        let any_sense_both = r.entry.sense.iter().any(|s| {
            s.part_of_speech.iter().any(|p| p.contains("v"))
                && s.misc.iter().any(|m| m == "uk")
        });
        assert!(
            any_sense_both,
            "entry {} kept under pos=v+misc=uk but no single sense has both",
            r.entry.id
        );
    }
}

// --- Xref resolution --------------------------------------------------------

#[test]
fn test_resolve_xref_term_only() {
    let dict = create_test_dict();
    let xref = Xref {
        term: "猫".to_string(),
        reading: None,
        sense_index: None,
    };
    let results = dict.resolve_xref(&xref);
    assert!(!results.is_empty());
    assert!(results.iter().any(|r| r.entry.kanji[0].text == "猫"));
}

#[test]
fn test_resolve_xref_with_reading_disambiguates() {
    let dict = create_test_dict();
    // 生 has multiple readings (なま, せい, etc.). With reading=なま we keep
    // only the matching entry.
    let xref = Xref {
        term: "生".to_string(),
        reading: Some("なま".to_string()),
        sense_index: None,
    };
    let results = dict.resolve_xref(&xref);
    for r in &results {
        assert!(
            r.entry.kana.iter().any(|k| k.text == "なま"),
            "entry {} kept under reading filter but no kana matches なま",
            r.entry.id
        );
    }
}

#[test]
fn test_resolve_xref_missing_term() {
    let dict = create_test_dict();
    let xref = Xref {
        term: "存在しない用語".to_string(),
        reading: None,
        sense_index: None,
    };
    assert!(dict.resolve_xref(&xref).is_empty());
}

#[test]
fn test_resolve_xref_walks_real_related_link() {
    let dict = create_test_dict();
    // Find any entry with a non-empty `related` xref, then resolve it.
    // Picks the first such link encountered during a bounded iter.
    let (entry, xref) = dict
        .iter_entries()
        .take(5_000)
        .find_map(|e| {
            e.sense
                .iter()
                .flat_map(|s| s.related.iter())
                .find(|x| !x.term.is_empty())
                .cloned()
                .map(|x| (e, x))
        })
        .expect("expected some entry in the first 5k with a related xref");

    let resolved = dict.resolve_xref(&xref);
    assert!(
        !resolved.is_empty(),
        "related xref {:?} from entry {} should resolve to at least one entry",
        xref,
        entry.id
    );
}

#[test]
fn test_entry_headword_falls_back_to_kana_when_no_kanji() {
    let dict = create_test_dict();
    let kana_only = dict
        .iter_entries()
        .find(|e| e.kanji.is_empty() && !e.kana.is_empty())
        .expect("expected at least one kana-only entry in JMdict");
    assert!(kana_only.primary_kanji().is_none());
    assert_eq!(kana_only.headword(), kana_only.primary_kana());
}
