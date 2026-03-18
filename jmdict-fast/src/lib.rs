mod error;

pub use error::JmdictError;

use fst::{automaton::Str, Automaton, IntoStreamer, Map, Streamer};
use memmap2::Mmap;
use postcard;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::{borrow::Cow, fs::File, path::Path};

/// How a lookup result matched the query term.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,
    Prefix,
    Deinflected,
    Fuzzy,
}

/// The search mode for a query.
#[derive(Debug, Clone, PartialEq)]
pub enum MatchMode {
    /// Exact match only.
    Exact,
    /// Prefix (starts-with) search.
    Prefix,
    /// Exact match with deinflection fallback.
    Deinflect,
    /// Fuzzy (approximate) match.
    Fuzzy,
}

/// Information about how a term was deinflected to find its base form.
#[derive(Debug, Clone)]
pub struct DeinflectionInfo {
    pub original_form: String,
    pub base_form: String,
    pub rules: Vec<String>,
}

/// A structured lookup result with metadata about how it matched.
#[derive(Debug, Clone)]
pub struct LookupResult {
    pub entry: Entry,
    pub match_type: MatchType,
    pub match_key: String,
    pub score: f64,
    pub deinflection: Option<DeinflectionInfo>,
}

/// Magic bytes at the start of entries.bin
const MAGIC: &[u8; 4] = b"JMDF";

/// Binary format version for entries.bin
pub const FORMAT_VERSION: u32 = 2;

/// Size of the entries.bin header (magic + version)
const HEADER_SIZE: usize = 8;

#[derive(Debug, Deserialize, Clone)]
pub struct Entry {
    pub id: String,
    pub kanji: Vec<KanjiEntry>,
    pub kana: Vec<KanaEntry>,
    pub sense: Vec<SenseEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KanjiEntry {
    pub common: bool,
    pub text: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KanaEntry {
    pub common: bool,
    pub text: String,
    pub tags: Vec<String>,
    #[serde(rename = "appliesToKanji")]
    pub applies_to_kanji: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Xref {
    pub term: String,
    pub reading: Option<String>,
    pub sense_index: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LanguageSource {
    pub lang: String,
    pub full: bool,
    pub wasei: bool,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SenseEntry {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: Vec<String>,
    #[serde(rename = "appliesToKanji")]
    pub applies_to_kanji: Vec<String>,
    #[serde(rename = "appliesToKana")]
    pub applies_to_kana: Vec<String>,
    pub related: Vec<Xref>,
    pub antonym: Vec<Xref>,
    pub field: Vec<String>,
    pub dialect: Vec<String>,
    pub misc: Vec<String>,
    pub info: Vec<String>,
    pub language_source: Vec<LanguageSource>,
    pub gloss: Vec<GlossEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GlossEntry {
    pub lang: String,
    pub gender: Option<String>,
    #[serde(rename = "type")]
    pub gloss_type: Option<String>,
    pub text: String,
}

pub struct Dict<'a> {
    pub entries_blob: Cow<'a, [u8]>,
    pub kana_fst: Map<Cow<'a, [u8]>>,
    pub kanji_fst: Map<Cow<'a, [u8]>>,
    pub romaji_fst: Map<Cow<'a, [u8]>>,
    pub id_fst: Map<Cow<'a, [u8]>>,
    deinflector: bunpo::deinflector::Deinflector,
}

/// Validate the entries.bin header (magic number and format version)
fn validate_entries_header(data: &[u8]) -> Result<(), JmdictError> {
    if data.len() < HEADER_SIZE {
        return Err(JmdictError::DataCorrupted);
    }
    if &data[0..4] != MAGIC {
        return Err(JmdictError::DataCorrupted);
    }
    let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    if version != FORMAT_VERSION {
        return Err(JmdictError::DataVersionMismatch {
            expected: FORMAT_VERSION,
            found: version,
        });
    }
    Ok(())
}

impl<'a> Dict<'a> {
    /// Construct a Dict from in-memory slices (e.g., embedded bytes)
    pub fn from_slices(
        entries: &'a [u8],
        kana_fst: &'a [u8],
        kanji_fst: &'a [u8],
        romaji_fst: &'a [u8],
        id_fst: &'a [u8],
    ) -> Result<Self, JmdictError> {
        validate_entries_header(entries)?;
        Ok(Self {
            entries_blob: Cow::Borrowed(entries),
            kana_fst: Map::new(Cow::Borrowed(kana_fst))?,
            kanji_fst: Map::new(Cow::Borrowed(kanji_fst))?,
            romaji_fst: Map::new(Cow::Borrowed(romaji_fst))?,
            id_fst: Map::new(Cow::Borrowed(id_fst))?,
            deinflector: bunpo::deinflector::Deinflector::new(),
        })
    }

    /// Load all FSTs and entries into memory (via mmap from files)
    pub fn load<P: AsRef<Path>>(base_dir: P) -> Result<Self, JmdictError> {
        let base = base_dir.as_ref();
        let entries_file = File::open(base.join("entries.bin"))?;
        let kana_file = File::open(base.join("kana.fst"))?;
        let kanji_file = File::open(base.join("kanji.fst"))?;
        let romaji_file = File::open(base.join("romaji.fst"))?;
        let id_file = File::open(base.join("id.fst"))?;
        unsafe {
            let entries_blob = Cow::Owned(Mmap::map(&entries_file)?[..].to_vec());
            let kana_fst = Cow::Owned(Mmap::map(&kana_file)?[..].to_vec());
            let kanji_fst = Cow::Owned(Mmap::map(&kanji_file)?[..].to_vec());
            let romaji_fst = Cow::Owned(Mmap::map(&romaji_file)?[..].to_vec());
            let id_fst = Cow::Owned(Mmap::map(&id_file)?[..].to_vec());

            validate_entries_header(&entries_blob)?;

            Ok(Dict {
                entries_blob,
                kana_fst: Map::new(kana_fst)?,
                kanji_fst: Map::new(kanji_fst)?,
                romaji_fst: Map::new(romaji_fst)?,
                id_fst: Map::new(id_fst)?,
                deinflector: bunpo::deinflector::Deinflector::new(),
            })
        }
    }

    #[cfg(feature = "embedded")]
    pub fn load_embedded() -> Result<Self, JmdictError> {
        let entries = include_bytes!(concat!(env!("OUT_DIR"), "/entries.bin"));
        let kana_fst = include_bytes!(concat!(env!("OUT_DIR"), "/kana.fst"));
        let kanji_fst = include_bytes!(concat!(env!("OUT_DIR"), "/kanji.fst"));
        let romaji_fst = include_bytes!(concat!(env!("OUT_DIR"), "/romaji.fst"));
        let id_fst = include_bytes!(concat!(env!("OUT_DIR"), "/id.fst"));

        Self::from_slices(entries, kana_fst, kanji_fst, romaji_fst, id_fst)
    }

    pub fn load_default() -> Result<Self, JmdictError> {
        #[cfg(feature = "embedded")]
        {
            if let Ok(dict) = Self::load_embedded() {
                return Ok(dict);
            }
        }

        // Try JMDICT_DATA env var first
        if let Ok(data_path) = std::env::var("JMDICT_DATA") {
            return Self::load(Path::new(&data_path));
        }

        // Try dist/ relative to current dir
        let dist = Path::new("dist");
        if dist.join("entries.bin").exists() {
            return Self::load(dist);
        }

        // Try dist/ relative to workspace root (for tests run from subdirectory)
        let workspace_dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist");
        if workspace_dist.join("entries.bin").exists() {
            return Self::load(&workspace_dist);
        }

        Self::load(dist)
    }

    /// Lookup a term exactly across kana, kanji, romaji.
    ///
    /// Convenience method equivalent to `dict.lookup(term).mode(MatchMode::Exact).execute()`.
    pub fn lookup_exact(&self, term: &str) -> Vec<LookupResult> {
        self.lookup_exact_inner(term)
    }

    fn lookup_exact_inner(&self, term: &str) -> Vec<LookupResult> {
        let mut ids = Vec::new();

        if let Some(id) = self.kana_fst.get(term) {
            ids.push(id);
        }
        if let Some(id) = self.kanji_fst.get(term) {
            ids.push(id);
        }
        if let Some(id) = self.romaji_fst.get(term) {
            ids.push(id);
        }

        ids.sort();
        ids.dedup();

        ids.into_iter()
            .filter_map(|id| {
                self.load_entry(id).map(|entry| LookupResult {
                    entry,
                    match_type: MatchType::Exact,
                    match_key: term.to_string(),
                    score: 1.0,
                    deinflection: None,
                })
            })
            .collect()
    }

    /// Lookup a term with deinflection fallback.
    ///
    /// Convenience method equivalent to `dict.lookup(term).mode(MatchMode::Deinflect).execute()`.
    pub fn lookup_exact_with_deinflection(&self, term: &str) -> Vec<LookupResult> {
        self.lookup_exact_with_deinflection_inner(term)
    }

    fn lookup_exact_with_deinflection_inner(&self, term: &str) -> Vec<LookupResult> {
        // First lookup exact
        let results = self.lookup_exact_inner(term);
        if !results.is_empty() {
            return results;
        }

        // Then deinflect
        let deinflected = self.deinflector.deinflect(term);
        let mut seen_ids = BTreeSet::new();
        let mut results = Vec::new();
        for candidate in deinflected {
            let exact = self.lookup_exact_inner(&candidate.word);
            for mut lr in exact {
                // Deduplicate by entry id
                if !seen_ids.insert(lr.entry.id.clone()) {
                    continue;
                }
                lr.match_type = MatchType::Deinflected;
                lr.match_key = candidate.word.clone();
                lr.score = 0.75;
                lr.deinflection = Some(DeinflectionInfo {
                    original_form: term.to_string(),
                    base_form: candidate.word.clone(),
                    rules: candidate
                        .reason_chains
                        .iter()
                        .flatten()
                        .map(|r| format!("{:?}", r))
                        .collect(),
                });
                results.push(lr);
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    /// Lookup entries that start with the given term (prefix search).
    ///
    /// Convenience method equivalent to `dict.lookup(term).mode(MatchMode::Prefix).execute()`.
    pub fn lookup_partial(&self, prefix: &str) -> Vec<LookupResult> {
        self.lookup_partial_inner(prefix)
    }

    fn lookup_partial_inner(&self, prefix: &str) -> Vec<LookupResult> {
        let mut id_keys: Vec<(u64, String)> = Vec::new();

        let automaton = Str::new(prefix).starts_with();

        for fst in [&self.kana_fst, &self.kanji_fst, &self.romaji_fst] {
            let mut stream = fst.search(&automaton).into_stream();
            while let Some((key, val)) = stream.next() {
                let key_str = String::from_utf8_lossy(key).to_string();
                id_keys.push((val, key_str));
            }
        }

        // Deduplicate by id, keeping first match key
        let mut seen = BTreeSet::new();
        let mut results = Vec::new();
        for (id, key) in id_keys {
            if !seen.insert(id) {
                continue;
            }
            if let Some(entry) = self.load_entry(id) {
                let is_exact = key == prefix;
                let score = if is_exact { 1.0 } else { 0.5 };
                let match_type = if is_exact {
                    MatchType::Exact
                } else {
                    MatchType::Prefix
                };
                results.push(LookupResult {
                    entry,
                    match_type,
                    match_key: key,
                    score,
                    deinflection: None,
                });
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }

    /// Create a query builder for the given term.
    pub fn lookup(&self, term: &str) -> QueryBuilder<'_, 'a> {
        QueryBuilder {
            dict: self,
            term: term.to_string(),
            mode: MatchMode::Exact,
        }
    }

    // When reading offsets, start after header (8 bytes) + entry_count (4 bytes)
    fn load_entry(&self, id: u64) -> Option<Entry> {
        let count = u32::from_le_bytes(
            self.entries_blob[HEADER_SIZE..HEADER_SIZE + 4]
                .try_into()
                .ok()?,
        ) as usize;
        if id as usize >= count {
            return None;
        }
        let offset_index = HEADER_SIZE + 4 + (id as usize) * 8;
        let off = u32::from_le_bytes(
            self.entries_blob[offset_index..offset_index + 4]
                .try_into()
                .ok()?,
        );
        let len = u32::from_le_bytes(
            self.entries_blob[offset_index + 4..offset_index + 8]
                .try_into()
                .ok()?,
        );

        let data_start = HEADER_SIZE + 4 + count * 8;
        let start = data_start + (off as usize);
        let end = start + len as usize;

        postcard::from_bytes(&self.entries_blob[start..end]).ok()
    }
}

/// A builder for configuring and executing dictionary lookups.
pub struct QueryBuilder<'d, 'a> {
    dict: &'d Dict<'a>,
    term: String,
    mode: MatchMode,
}

impl<'d, 'a> QueryBuilder<'d, 'a> {
    /// Set the match mode for this query.
    pub fn mode(mut self, mode: MatchMode) -> Self {
        self.mode = mode;
        self
    }

    /// Execute the query and return results.
    pub fn execute(self) -> Result<Vec<LookupResult>, JmdictError> {
        let results = match self.mode {
            MatchMode::Exact => self.dict.lookup_exact_inner(&self.term),
            MatchMode::Prefix => self.dict.lookup_partial_inner(&self.term),
            MatchMode::Deinflect => self.dict.lookup_exact_with_deinflection_inner(&self.term),
            MatchMode::Fuzzy => {
                // Fuzzy search will be implemented in US-013
                Vec::new()
            }
        };
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
