use anyhow::{bail, Result};
use fst::{automaton::Str, Automaton, IntoStreamer, Map, Streamer};
use memmap2::Mmap;
use postcard;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::{borrow::Cow, fs::File, path::Path};

/// Magic bytes at the start of entries.bin
const MAGIC: &[u8; 4] = b"JMDF";

/// Binary format version for entries.bin
pub const FORMAT_VERSION: u32 = 1;

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
pub struct SenseEntry {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: Vec<String>,
    #[serde(rename = "appliesToKanji")]
    pub applies_to_kanji: Vec<String>,
    #[serde(rename = "appliesToKana")]
    pub applies_to_kana: Vec<String>,
    pub related: Vec<serde_json::Value>,
    pub antonym: Vec<serde_json::Value>,
    pub field: Vec<String>,
    pub dialect: Vec<String>,
    pub misc: Vec<String>,
    pub info: Vec<String>,
    #[serde(rename = "languageSource")]
    pub language_source: Vec<serde_json::Value>,
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
fn validate_entries_header(data: &[u8]) -> Result<()> {
    if data.len() < HEADER_SIZE {
        bail!("entries.bin is too small to contain a valid header");
    }
    if &data[0..4] != MAGIC {
        bail!("entries.bin has invalid magic number. Regenerate with `cargo xtask generate`.");
    }
    let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    if version != FORMAT_VERSION {
        bail!(
            "Data format version {}, library expects {}. Regenerate with cargo xtask generate.",
            version,
            FORMAT_VERSION
        );
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
    ) -> Result<Self> {
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
    pub fn load<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
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
    pub fn load_embedded() -> Result<Self> {
        let entries = include_bytes!(concat!(env!("OUT_DIR"), "/entries.bin"));
        let kana_fst = include_bytes!(concat!(env!("OUT_DIR"), "/kana.fst"));
        let kanji_fst = include_bytes!(concat!(env!("OUT_DIR"), "/kanji.fst"));
        let romaji_fst = include_bytes!(concat!(env!("OUT_DIR"), "/romaji.fst"));
        let id_fst = include_bytes!(concat!(env!("OUT_DIR"), "/id.fst"));

        Self::from_slices(entries, kana_fst, kanji_fst, romaji_fst, id_fst)
    }

    pub fn load_default() -> Result<Self> {
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

    /// Lookup a term exactly across kana, kanji, romaji
    pub fn lookup_exact(&self, term: &str) -> Vec<Entry> {
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
            .filter_map(|id| self.load_entry(id))
            .collect()
    }

    pub fn lookup_exact_with_deinflection(&self, term: &str) -> Vec<Entry> {
        // First lookup exact
        let results = self.lookup_exact(term);
        if !results.is_empty() {
            return results;
        }

        // Then deinflect
        let deinflected = self.deinflector.deinflect(term);
        let mut results = Vec::new();
        for candidate in deinflected {
            results.extend(self.lookup_exact(&candidate.word));
        }
        results
    }

    /// Lookup entries that start with the given term (prefix search)
    pub fn lookup_partial(&self, prefix: &str) -> Vec<Entry> {
        // sorted + dedup
        let mut ids = BTreeSet::new();

        let automaton = Str::new(prefix).starts_with();

        for fst in [&self.kana_fst, &self.kanji_fst, &self.romaji_fst] {
            let mut stream = fst.search(&automaton).into_stream();
            while let Some((_key, val)) = stream.next() {
                ids.insert(val);
            }
        }

        ids.into_iter()
            .filter_map(|id| self.load_entry(id))
            .collect()
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
        assert_eq!(results[0].kana[0].text, "ねこ");
        assert_eq!(results[0].kanji[0].text, "猫");
        assert_eq!(
            results[0].sense[0].gloss[0].text,
            "cat (esp. the domestic cat, Felis catus)"
        );
    }

    #[test]
    fn test_lookup_exact_verb() {
        let dict = create_test_dict();

        let results = dict.lookup_exact("食べる");

        assert!(!results.is_empty(), "Expected to find entries for 食べる");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kana[0].text, "たべる");
        assert_eq!(results[0].kanji[0].text, "食べる");
        assert_eq!(results[0].sense[0].gloss[0].text, "to eat");
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
        assert_eq!(results[0].kana[0].text, "たべる");
        assert_eq!(results[0].kanji[0].text, "食べる");

        let results = dict.lookup_exact_with_deinflection("食べます");
        assert!(!results.is_empty(), "Expected to find entries for 食べます");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kana[0].text, "たべる");
        assert_eq!(results[0].kanji[0].text, "食べる");
    }

    #[test]
    fn test_lookup_exact_with_deinflection_adj() {
        let dict = create_test_dict();
        let results = dict.lookup_exact_with_deinflection("美しい");
        assert!(!results.is_empty(), "Expected to find entries for 美しい");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kana[0].text, "うつくしい");
        assert_eq!(results[0].kanji[0].text, "美しい");
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
            results.iter().any(|r| r.kana[0].text == "たべる"),
            "Expected to find たべる in results"
        );
        assert!(
            results.iter().any(|r| r.kanji[0].text == "食べる"),
            "Expected to find 食べる in results"
        );
        assert!(
            results.iter().any(|r| r.sense[0].gloss[0].text == "to eat"),
            "Expected to find to eat in results"
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
            .any(|entry| entry.kana.iter().any(|k| k.text.starts_with("ね")));
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
    }
}
