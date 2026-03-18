mod error;

pub use error::JmdictError;

use fst::{automaton::Levenshtein, automaton::Str, Automaton, IntoStreamer, Map, Streamer};
use memmap2::Mmap;
use postcard;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::{borrow::Cow, fs::File, path::Path, vec};

/// A raw match candidate from FST search, before entry deserialization.
#[derive(Clone)]
struct MatchCandidate {
    id: u64,
    key: String,
    match_type: MatchType,
    score: f64,
    deinflection: Option<DeinflectionInfo>,
}

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
pub const FORMAT_VERSION: u32 = 3;

/// Dictionary data version information.
#[derive(Debug, Clone)]
pub struct DataVersion {
    pub format_version: u32,
    pub jmdict_version: String,
    pub generated_at: String,
}

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
    data_version: DataVersion,
    header_size: usize,
}

/// Parsed header information from entries.bin
struct HeaderInfo {
    data_version: DataVersion,
    /// Total bytes before entry_count (magic + version + metadata strings)
    header_size: usize,
}

/// Parse the entries.bin header: magic, format version, jmdict_version, generated_at
fn parse_entries_header(data: &[u8]) -> Result<HeaderInfo, JmdictError> {
    if data.len() < 8 {
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

    // Parse jmdict_version (u16 len + bytes)
    if data.len() < 10 {
        return Err(JmdictError::DataCorrupted);
    }
    let jmdict_ver_len = u16::from_le_bytes(data[8..10].try_into().unwrap()) as usize;
    let mut pos = 10;
    if data.len() < pos + jmdict_ver_len + 2 {
        return Err(JmdictError::DataCorrupted);
    }
    let jmdict_version = String::from_utf8_lossy(&data[pos..pos + jmdict_ver_len]).to_string();
    pos += jmdict_ver_len;

    // Parse generated_at (u16 len + bytes)
    let gen_at_len = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
    pos += 2;
    if data.len() < pos + gen_at_len {
        return Err(JmdictError::DataCorrupted);
    }
    let generated_at = String::from_utf8_lossy(&data[pos..pos + gen_at_len]).to_string();
    pos += gen_at_len;

    Ok(HeaderInfo {
        data_version: DataVersion {
            format_version: version,
            jmdict_version,
            generated_at,
        },
        header_size: pos,
    })
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
        let header = parse_entries_header(entries)?;
        Ok(Self {
            entries_blob: Cow::Borrowed(entries),
            kana_fst: Map::new(Cow::Borrowed(kana_fst))?,
            kanji_fst: Map::new(Cow::Borrowed(kanji_fst))?,
            romaji_fst: Map::new(Cow::Borrowed(romaji_fst))?,
            id_fst: Map::new(Cow::Borrowed(id_fst))?,
            deinflector: bunpo::deinflector::Deinflector::new(),
            data_version: header.data_version,
            header_size: header.header_size,
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

            let header = parse_entries_header(&entries_blob)?;

            Ok(Dict {
                entries_blob,
                kana_fst: Map::new(kana_fst)?,
                kanji_fst: Map::new(kanji_fst)?,
                romaji_fst: Map::new(romaji_fst)?,
                id_fst: Map::new(id_fst)?,
                deinflector: bunpo::deinflector::Deinflector::new(),
                data_version: header.data_version,
                header_size: header.header_size,
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

    /// Returns the total number of entries in the dictionary.
    pub fn entry_count(&self) -> usize {
        u32::from_le_bytes(
            self.entries_blob[self.header_size..self.header_size + 4]
                .try_into()
                .unwrap(),
        ) as usize
    }

    /// Returns data version information (format version, JMdict source version, generation timestamp).
    pub fn version(&self) -> DataVersion {
        self.data_version.clone()
    }

    /// Lookup a term exactly across kana, kanji, romaji.
    ///
    /// Convenience method equivalent to `dict.lookup(term).mode(MatchMode::Exact).execute()`.
    pub fn lookup_exact(&self, term: &str) -> Vec<LookupResult> {
        self.lookup_exact_inner(term)
    }

    fn lookup_exact_inner(&self, term: &str) -> Vec<LookupResult> {
        self.candidates_to_results(self.exact_candidates(term))
    }

    fn exact_candidates(&self, term: &str) -> Vec<MatchCandidate> {
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
            .map(|id| MatchCandidate {
                id,
                key: term.to_string(),
                match_type: MatchType::Exact,
                score: 1.0,
                deinflection: None,
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
        self.candidates_to_results(self.deinflect_candidates(term))
    }

    fn deinflect_candidates(&self, term: &str) -> Vec<MatchCandidate> {
        // First try exact
        let exact = self.exact_candidates(term);
        if !exact.is_empty() {
            return exact;
        }

        // Then deinflect
        let deinflected = self.deinflector.deinflect(term);
        let mut seen_ids = BTreeSet::new();
        let mut candidates = Vec::new();
        for candidate in deinflected {
            let exact = self.exact_candidates(&candidate.word);
            for mc in exact {
                if !seen_ids.insert(mc.id) {
                    continue;
                }
                candidates.push(MatchCandidate {
                    id: mc.id,
                    key: candidate.word.clone(),
                    match_type: MatchType::Deinflected,
                    score: 0.75,
                    deinflection: Some(DeinflectionInfo {
                        original_form: term.to_string(),
                        base_form: candidate.word.clone(),
                        rules: candidate
                            .reason_chains
                            .iter()
                            .flatten()
                            .map(|r| format!("{:?}", r))
                            .collect(),
                    }),
                });
            }
        }

        // Sort by score descending
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        candidates
    }

    /// Lookup entries that start with the given term (prefix search).
    ///
    /// Convenience method equivalent to `dict.lookup(term).mode(MatchMode::Prefix).execute()`.
    pub fn lookup_partial(&self, prefix: &str) -> Vec<LookupResult> {
        self.lookup_partial_inner(prefix)
    }

    fn lookup_partial_inner(&self, prefix: &str) -> Vec<LookupResult> {
        self.candidates_to_results(self.prefix_candidates(prefix))
    }

    fn prefix_candidates(&self, prefix: &str) -> Vec<MatchCandidate> {
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
        let mut candidates = Vec::new();
        for (id, key) in id_keys {
            if !seen.insert(id) {
                continue;
            }
            let is_exact = key == prefix;
            let score = if is_exact { 1.0 } else { 0.5 };
            let match_type = if is_exact {
                MatchType::Exact
            } else {
                MatchType::Prefix
            };
            candidates.push(MatchCandidate {
                id,
                key,
                match_type,
                score,
                deinflection: None,
            });
        }

        // Sort by score descending
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        candidates
    }

    fn fuzzy_candidates(&self, term: &str, max_distance: u32) -> Result<Vec<MatchCandidate>, JmdictError> {
        let automaton = Levenshtein::new(term, max_distance)
            .map_err(|_| JmdictError::InvalidQuery)?;

        let mut id_keys: Vec<(u64, String)> = Vec::new();

        for fst in [&self.kana_fst, &self.kanji_fst, &self.romaji_fst] {
            let mut stream = fst.search(&automaton).into_stream();
            while let Some((key, val)) = stream.next() {
                let key_str = String::from_utf8_lossy(key).to_string();
                id_keys.push((val, key_str));
            }
        }

        // Deduplicate by id
        let mut seen = BTreeSet::new();
        let mut candidates = Vec::new();
        for (id, key) in id_keys {
            if !seen.insert(id) {
                continue;
            }
            let is_exact = key == term;
            let (match_type, score) = if is_exact {
                (MatchType::Exact, 1.0)
            } else {
                let key_len = key.chars().count().max(1) as f64;
                let term_len = term.chars().count().max(1) as f64;
                let len_diff = (key_len - term_len).abs();
                let score = 0.5 - (len_diff / (key_len + term_len)) * 0.2;
                (MatchType::Fuzzy, score.max(0.1))
            };
            candidates.push(MatchCandidate {
                id,
                key,
                match_type,
                score,
                deinflection: None,
            });
        }

        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(candidates)
    }

    /// Create a query builder for the given term.
    pub fn lookup(&self, term: &str) -> QueryBuilder<'_, 'a> {
        QueryBuilder {
            dict: self,
            term: term.to_string(),
            mode: MatchMode::Exact,
            common_only: false,
            pos_filter: Vec::new(),
            limit: None,
            max_distance: 2,
        }
    }

    /// Create a batch query builder for multiple terms.
    pub fn lookup_batch(&self, terms: &[&str]) -> BatchQueryBuilder<'_, 'a> {
        BatchQueryBuilder {
            dict: self,
            terms: terms.iter().map(|s| s.to_string()).collect(),
            mode: MatchMode::Exact,
            common_only: false,
            pos_filter: Vec::new(),
            limit: None,
            max_distance: 2,
        }
    }

    /// Convert match candidates to results by deserializing entries.
    fn candidates_to_results(&self, candidates: Vec<MatchCandidate>) -> Vec<LookupResult> {
        candidates
            .into_iter()
            .filter_map(|mc| {
                self.load_entry(mc.id).map(|entry| LookupResult {
                    entry,
                    match_type: mc.match_type,
                    match_key: mc.key,
                    score: mc.score,
                    deinflection: mc.deinflection,
                })
            })
            .collect()
    }

    // When reading offsets, start after header + entry_count (4 bytes)
    fn load_entry(&self, id: u64) -> Option<Entry> {
        let hs = self.header_size;
        let count = u32::from_le_bytes(
            self.entries_blob[hs..hs + 4].try_into().ok()?,
        ) as usize;
        if id as usize >= count {
            return None;
        }
        let offset_index = hs + 4 + (id as usize) * 8;
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

        let data_start = hs + 4 + count * 8;
        let start = data_start + (off as usize);
        let end = start + len as usize;

        postcard::from_bytes(&self.entries_blob[start..end]).ok()
    }
}

/// An iterator that lazily deserializes dictionary entries from pre-sorted match candidates.
pub struct LookupResultIter<'d, 'a> {
    dict: &'d Dict<'a>,
    candidates: vec::IntoIter<MatchCandidate>,
    common_only: bool,
    pos_filter: Vec<String>,
    limit: Option<usize>,
    yielded: usize,
}

impl<'d, 'a> Iterator for LookupResultIter<'d, 'a> {
    type Item = LookupResult;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(limit) = self.limit {
            if self.yielded >= limit {
                return None;
            }
        }

        loop {
            let mc = self.candidates.next()?;
            let entry = match self.dict.load_entry(mc.id) {
                Some(e) => e,
                None => continue,
            };

            if self.common_only {
                let is_common = entry.kanji.iter().any(|k| k.common)
                    || entry.kana.iter().any(|k| k.common);
                if !is_common {
                    continue;
                }
            }

            if !self.pos_filter.is_empty() {
                let matches_pos = entry.sense.iter().any(|s| {
                    s.part_of_speech
                        .iter()
                        .any(|p| self.pos_filter.iter().any(|f| p.contains(f.as_str())))
                });
                if !matches_pos {
                    continue;
                }
            }

            self.yielded += 1;
            return Some(LookupResult {
                entry,
                match_type: mc.match_type,
                match_key: mc.key,
                score: mc.score,
                deinflection: mc.deinflection,
            });
        }
    }
}

/// A builder for configuring and executing dictionary lookups.
pub struct QueryBuilder<'d, 'a> {
    dict: &'d Dict<'a>,
    term: String,
    mode: MatchMode,
    common_only: bool,
    pos_filter: Vec<String>,
    limit: Option<usize>,
    max_distance: u32,
}

impl<'d, 'a> QueryBuilder<'d, 'a> {
    /// Set the match mode for this query.
    pub fn mode(mut self, mode: MatchMode) -> Self {
        self.mode = mode;
        self
    }

    /// Filter to entries where any KanjiEntry or KanaEntry has `common: true`.
    pub fn common_only(mut self, common: bool) -> Self {
        self.common_only = common;
        self
    }

    /// Filter to entries with matching part_of_speech values in any SenseEntry.
    pub fn pos(mut self, pos: &[&str]) -> Self {
        self.pos_filter = pos.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set the maximum edit distance for fuzzy search (default: 2).
    pub fn max_distance(mut self, n: u32) -> Self {
        self.max_distance = n;
        self
    }

    /// Cap results after filtering and sorting.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Execute the query and return all results collected into a Vec.
    pub fn execute(self) -> Result<Vec<LookupResult>, JmdictError> {
        Ok(self.execute_iter()?.collect())
    }

    /// Execute the query and return a lazy iterator that deserializes entries on demand.
    ///
    /// This is more memory-efficient than `execute()` for large result sets (e.g., prefix
    /// or fuzzy queries with many matches), as entries are only deserialized as consumed.
    pub fn execute_iter(self) -> Result<LookupResultIter<'d, 'a>, JmdictError> {
        let candidates = match self.mode {
            MatchMode::Exact => self.dict.exact_candidates(&self.term),
            MatchMode::Prefix => self.dict.prefix_candidates(&self.term),
            MatchMode::Deinflect => self.dict.deinflect_candidates(&self.term),
            MatchMode::Fuzzy => self.dict.fuzzy_candidates(&self.term, self.max_distance)?,
        };

        Ok(LookupResultIter {
            dict: self.dict,
            candidates: candidates.into_iter(),
            common_only: self.common_only,
            pos_filter: self.pos_filter,
            limit: self.limit,
            yielded: 0,
        })
    }
}

/// A builder for configuring and executing batch dictionary lookups.
pub struct BatchQueryBuilder<'d, 'a> {
    dict: &'d Dict<'a>,
    terms: Vec<String>,
    mode: MatchMode,
    common_only: bool,
    pos_filter: Vec<String>,
    limit: Option<usize>,
    max_distance: u32,
}

impl<'d, 'a> BatchQueryBuilder<'d, 'a> {
    /// Set the match mode for this batch query.
    pub fn mode(mut self, mode: MatchMode) -> Self {
        self.mode = mode;
        self
    }

    /// Filter to entries where any KanjiEntry or KanaEntry has `common: true`.
    pub fn common_only(mut self, common: bool) -> Self {
        self.common_only = common;
        self
    }

    /// Filter to entries with matching part_of_speech values in any SenseEntry.
    pub fn pos(mut self, pos: &[&str]) -> Self {
        self.pos_filter = pos.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Cap results per term after filtering and sorting.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the maximum edit distance for fuzzy search (default: 2).
    pub fn max_distance(mut self, n: u32) -> Self {
        self.max_distance = n;
        self
    }

    /// Execute the batch query and return results paired with each input term.
    pub fn execute(self) -> Result<Vec<(String, Vec<LookupResult>)>, JmdictError> {
        let pos_refs: Vec<&str> = self.pos_filter.iter().map(|s| s.as_str()).collect();
        let mut batch_results = Vec::with_capacity(self.terms.len());
        for term in &self.terms {
            let mut builder = self
                .dict
                .lookup(term)
                .mode(self.mode.clone())
                .common_only(self.common_only)
                .pos(&pos_refs)
                .max_distance(self.max_distance);
            if let Some(limit) = self.limit {
                builder = builder.limit(limit);
            }
            batch_results.push((term.clone(), builder.execute()?));
        }
        Ok(batch_results)
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
}
