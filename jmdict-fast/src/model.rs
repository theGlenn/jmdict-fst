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

impl Entry {
    /// The first kanji writing, if any. Many entries (kana-only words, names)
    /// have no kanji form, so callers should not assume `kanji[0]` exists.
    pub fn primary_kanji(&self) -> Option<&str> {
        self.kanji.first().map(|k| k.text.as_str())
    }

    /// The first kana reading. Every JMdict entry has at least one kana
    /// reading, but the field is technically a `Vec`, so this returns `Option`
    /// to stay honest about the data model.
    pub fn primary_kana(&self) -> Option<&str> {
        self.kana.first().map(|k| k.text.as_str())
    }

    /// The preferred headword for display: kanji if present, otherwise kana.
    pub fn headword(&self) -> Option<&str> {
        self.primary_kanji().or_else(|| self.primary_kana())
    }

    /// `true` if any kanji or kana form is marked common by JMdict.
    pub fn is_common(&self) -> bool {
        self.kanji.iter().any(|k| k.common) || self.kana.iter().any(|k| k.common)
    }

    /// Iterate over every gloss matching `lang` (e.g. `"eng"`), across all senses.
    pub fn glosses<'a>(&'a self, lang: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.sense
            .iter()
            .flat_map(move |s| s.gloss.iter())
            .filter(move |g| g.lang == lang)
            .map(|g| g.text.as_str())
    }

    /// Every distinct part-of-speech tag across all senses, preserving first-seen order.
    pub fn parts_of_speech(&self) -> Vec<&str> {
        let mut seen = Vec::new();
        for s in &self.sense {
            for p in &s.part_of_speech {
                if !seen.iter().any(|x: &&str| *x == p.as_str()) {
                    seen.push(p.as_str());
                }
            }
        }
        seen
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn gloss(lang: &str, text: &str) -> GlossEntry {
        GlossEntry {
            lang: lang.into(),
            gender: None,
            gloss_type: None,
            text: text.into(),
        }
    }

    fn sense(pos: &[&str], glosses: Vec<GlossEntry>) -> SenseEntry {
        SenseEntry {
            part_of_speech: pos.iter().map(|s| s.to_string()).collect(),
            applies_to_kanji: Vec::new(),
            applies_to_kana: Vec::new(),
            related: Vec::new(),
            antonym: Vec::new(),
            field: Vec::new(),
            dialect: Vec::new(),
            misc: Vec::new(),
            info: Vec::new(),
            language_source: Vec::new(),
            gloss: glosses,
        }
    }

    fn kanji(text: &str, common: bool) -> KanjiEntry {
        KanjiEntry {
            common,
            text: text.into(),
            tags: Vec::new(),
        }
    }

    fn kana(text: &str, common: bool) -> KanaEntry {
        KanaEntry {
            common,
            text: text.into(),
            tags: Vec::new(),
            applies_to_kanji: Vec::new(),
        }
    }

    fn entry(kanji: Vec<KanjiEntry>, kana: Vec<KanaEntry>, sense: Vec<SenseEntry>) -> Entry {
        Entry {
            id: "test".into(),
            kanji,
            kana,
            sense,
        }
    }

    #[test]
    fn primary_kanji_and_kana_first_element() {
        let e = entry(
            vec![kanji("猫", true), kanji("ねこ", false)],
            vec![kana("ねこ", true)],
            vec![],
        );
        assert_eq!(e.primary_kanji(), Some("猫"));
        assert_eq!(e.primary_kana(), Some("ねこ"));
    }

    #[test]
    fn primary_kanji_none_when_kana_only() {
        let e = entry(vec![], vec![kana("にゃんこ", false)], vec![]);
        assert!(e.primary_kanji().is_none());
        assert_eq!(e.primary_kana(), Some("にゃんこ"));
    }

    #[test]
    fn headword_prefers_kanji_falls_back_to_kana() {
        let with_kanji = entry(vec![kanji("猫", false)], vec![kana("ねこ", false)], vec![]);
        assert_eq!(with_kanji.headword(), Some("猫"));

        let kana_only = entry(vec![], vec![kana("にゃんこ", false)], vec![]);
        assert_eq!(kana_only.headword(), Some("にゃんこ"));

        let empty = entry(vec![], vec![], vec![]);
        assert!(empty.headword().is_none());
    }

    #[test]
    fn is_common_true_if_any_form_is_common() {
        let kanji_common = entry(vec![kanji("猫", true)], vec![kana("ねこ", false)], vec![]);
        assert!(kanji_common.is_common());

        let kana_common = entry(vec![kanji("猫", false)], vec![kana("ねこ", true)], vec![]);
        assert!(kana_common.is_common());

        let neither = entry(vec![kanji("猫", false)], vec![kana("ねこ", false)], vec![]);
        assert!(!neither.is_common());
    }

    #[test]
    fn glosses_filter_by_lang() {
        let e = entry(
            vec![],
            vec![kana("ねこ", false)],
            vec![sense(
                &["n"],
                vec![gloss("eng", "cat"), gloss("fre", "chat"), gloss("eng", "feline")],
            )],
        );
        let eng: Vec<&str> = e.glosses("eng").collect();
        assert_eq!(eng, vec!["cat", "feline"]);

        let fre: Vec<&str> = e.glosses("fre").collect();
        assert_eq!(fre, vec!["chat"]);

        let missing: Vec<&str> = e.glosses("jpn").collect();
        assert!(missing.is_empty());
    }

    #[test]
    fn parts_of_speech_dedup_in_first_seen_order() {
        let e = entry(
            vec![],
            vec![kana("ねこ", false)],
            vec![
                sense(&["v1", "vt"], vec![]),
                sense(&["vt", "vi"], vec![]),
                sense(&["v1"], vec![]),
            ],
        );
        assert_eq!(e.parts_of_speech(), vec!["v1", "vt", "vi"]);
    }
}
