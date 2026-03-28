use crate::error::JmdictError;
use crate::model::{
    DataVersion, DeinflectionInfo, Entry, LookupResult, MatchType, FORMAT_VERSION,
};
use crate::query::{BatchQueryBuilder, QueryBuilder};
use fst::{automaton::Levenshtein, automaton::Str, Automaton, IntoStreamer, Map, Streamer};
use memmap2::Mmap;
use std::collections::BTreeSet;
use std::{borrow::Cow, fs::File, path::Path};

/// A raw match candidate from FST search, before entry deserialization.
#[derive(Clone)]
pub(crate) struct MatchCandidate {
    pub(crate) id: u64,
    pub(crate) key: String,
    pub(crate) match_type: MatchType,
    pub(crate) score: f64,
    pub(crate) deinflection: Option<DeinflectionInfo>,
}

/// Magic bytes at the start of entries.bin
const MAGIC: &[u8; 4] = b"JMDF";

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

    pub(crate) fn exact_candidates(&self, term: &str) -> Vec<MatchCandidate> {
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

    pub(crate) fn deinflect_candidates(&self, term: &str) -> Vec<MatchCandidate> {
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

    pub(crate) fn prefix_candidates(&self, prefix: &str) -> Vec<MatchCandidate> {
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

    pub(crate) fn fuzzy_candidates(
        &self,
        term: &str,
        max_distance: u32,
    ) -> Result<Vec<MatchCandidate>, JmdictError> {
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
        QueryBuilder::new(self, term)
    }

    /// Create a batch query builder for multiple terms.
    pub fn lookup_batch(&self, terms: &[&str]) -> BatchQueryBuilder<'_, 'a> {
        BatchQueryBuilder::new(self, terms.iter().map(|s| s.to_string()).collect())
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
    pub(crate) fn load_entry(&self, id: u64) -> Option<Entry> {
        let hs = self.header_size;
        let count =
            u32::from_le_bytes(self.entries_blob[hs..hs + 4].try_into().ok()?) as usize;
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
