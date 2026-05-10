use crate::*;

#[test]
#[cfg(feature = "embedded")]
fn test_load_dict_embedded() {
    let dict = Dict::load_embedded().expect("load failed");
    assert!(dict.kana_fst.contains_key("ねこ"));
    assert!(dict.kanji_fst.contains_key("猫"));
    assert!(dict.romaji_fst.contains_key("neko"));

    assert!(dict.kana_fst.contains_key("たべる"));
    assert!(dict.kanji_fst.contains_key("食べる"));

    // uncommon kana
    assert!(dict.kana_fst.contains_key("にゃんこ"));
    // uncommon kanji
    assert!(dict.kanji_fst.contains_key("鯉"));
}

fn create_test_dict() -> Dict<'static> {
    Dict::load_default().expect("load failed")
}

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

    // We cannot find 食べます without deinflection
    assert!(
        results.is_empty(),
        "Expected to not find entries for 食べます"
    );
    assert_eq!(results.len(), 0);
}

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

#[test]
fn test_lookup_exact_multiple() {
    let dict = create_test_dict();

    let test_words = ["猫", "犬", "魚", "鳥", "花"];

    for word in test_words {
        let results = dict.lookup_exact(word);
        assert!(!results.is_empty(), "Expected to find entries for {}", word);
    }
}

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
    // Verify prefix results have appropriate match types
    assert!(
        results.iter().any(|r| r.match_type == MatchType::Prefix),
        "Expected some Prefix match types"
    );
}

#[test]
fn test_lookup_partial_in_depth() {
    let dict = create_test_dict();

    // Test partial lookup - should find entries starting with the term
    let partial_results = dict.lookup_partial("ね");
    assert!(
        !partial_results.is_empty(),
        "Partial lookup should find entries starting with ね"
    );

    // Verify that partial results include entries that start with "ね"
    let has_nekko = partial_results
        .iter()
        .any(|lr| lr.entry.kana.iter().any(|k| k.text.starts_with("ね")));
    assert!(
        has_nekko,
        "Partial results should include entries starting with ね"
    );

    // Test that partial lookup finds more results than exact for a prefix
    let exact_neko = dict.lookup_exact("ねこ");
    let partial_neko = dict.lookup_partial("ねこ");
    assert!(
        partial_neko.len() >= exact_neko.len(),
        "Partial lookup should find at least as many results as exact lookup"
    );

    // Verify results are sorted by score descending
    for window in partial_neko.windows(2) {
        assert!(
            window[0].score >= window[1].score,
            "Results should be sorted by score descending"
        );
    }
}

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
    // Without setting mode, should default to Exact
    let builder_results = dict.lookup("猫").execute().unwrap();
    let direct_results = dict.lookup_exact("猫");
    assert_eq!(builder_results.len(), direct_results.len());
    assert_eq!(builder_results[0].entry.id, direct_results[0].entry.id);
}

#[test]
fn test_query_builder_common_only() {
    let dict = create_test_dict();
    // 猫 (cat) is a common word
    let results = dict
        .lookup("猫")
        .common_only(true)
        .execute()
        .unwrap();
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
    // Common-only should return fewer or equal results
    assert!(common_results.len() <= all_results.len());
    // All common results should have at least one common reading
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
    // 食べる has POS codes like "v1", "vt"
    let results = dict
        .lookup("食べる")
        .pos(&["v1"])
        .execute()
        .unwrap();
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
    let results = dict
        .lookup("猫")
        .pos(&["v1"])
        .execute()
        .unwrap();
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
    // Chain all filters together — use "v1" POS code (ichidan verb)
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
    // All fuzzy (non-exact) results should have score < 1.0
    for r in &results {
        if r.match_type == MatchType::Fuzzy {
            assert!(r.score < 1.0, "Fuzzy results should have score < 1.0");
        }
    }
}

#[test]
fn test_fuzzy_search_exact_match_included() {
    let dict = create_test_dict();
    // "neko" should still return an exact match within fuzzy results
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
    // With distance 0, should only get exact matches
    let results_d0 = dict
        .lookup("neko")
        .mode(MatchMode::Fuzzy)
        .max_distance(0)
        .execute()
        .unwrap();
    for r in &results_d0 {
        assert_eq!(r.match_type, MatchType::Exact, "Distance 0 should only return exact matches");
    }

    // With distance 2, should get more results than distance 1
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
    // Fuzzy search with common_only filter
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
    // Each term should have results
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
    // 猫 is a noun, should have results
    assert!(!results[0].1.is_empty(), "猫 should match noun filter");
    // 食べる is a verb, should be filtered out by noun POS
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

    // Batch results should match individual lookups
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
    // Take only 2 from a prefix query that has many results
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
    // Only take first result from iterator — rest should not be deserialized
    let mut iter = dict
        .lookup("たべ")
        .mode(MatchMode::Prefix)
        .execute_iter()
        .unwrap();
    let first = iter.next();
    assert!(first.is_some(), "Should have at least one result");
    // Iterator still has more
    let second = iter.next();
    assert!(second.is_some(), "Should have more than one result for prefix たべ");
}

#[test]
fn test_entry_count() {
    let dict = create_test_dict();
    let count = dict.entry_count();
    // JMdict has tens of thousands of entries
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
