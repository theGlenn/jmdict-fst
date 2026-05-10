use serde::Deserialize;

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

/// Magic bytes that prefix every `entries.bin`. The library uses these to
/// distinguish a valid jmdict-fast data file from arbitrary input before
/// touching the format version.
pub const MAGIC: &[u8; 4] = b"JMDF";

/// Binary format version for entries.bin. Bump whenever the on-disk layout or
/// the serialized `Entry` struct changes — `Dict::load` rejects mismatched
/// versions instead of attempting a deserialize that may silently succeed.
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
