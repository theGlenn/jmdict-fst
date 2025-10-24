use serde::{Deserialize, Serialize};

/*
const DEINFLECTABLE_POS: &[&str] = &[
    // Verbs
    "v1",        // Ichidan verbs
    "v5u", "v5k", "v5g", "v5s", "v5t", "v5n", "v5b", "v5m", "v5r", "v5aru", // Godan verbs
    "vk",        // Kuru verb
    "vs",        // Suru verb
    "vs-i",      // Suru verb (irregular)
    "vz",        // Suru verb (archaic)

    // Adjectives
    "adj-i",     // I-adjectives
    "adj-ix",    // I-adjective (yoi/ii)

    // Special cases
    "vn",        // Noun + suru
    "vs-c",      // Suru verb (compound)
];
*/

// Follow https://www.edrdg.org/jmdict/jmdict_dtd_h.html for the schema

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Entry {
    pub id: String,
    pub kanji: Vec<KanjiEntry>,
    pub kana: Vec<KanaEntry>,
    pub sense: Vec<SenseEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KanjiEntry {
    pub common: bool,
    pub text: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KanaEntry {
    pub common: bool,
    pub text: String,
    pub tags: Vec<String>,
    #[serde(rename = "appliesToKanji")]
    pub applies_to_kanji: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SenseEntry {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: Vec<String>,
    #[serde(rename = "appliesToKanji")]
    pub applies_to_kanji: Vec<String>,
    #[serde(rename = "appliesToKana")]
    pub applies_to_kana: Vec<String>,
    pub related: Vec<serde_json::Value>, // Can be [String] or [String, Number]
    pub antonym: Vec<serde_json::Value>, // Can be String or [String]
    pub field: Vec<String>,
    pub dialect: Vec<String>,
    pub misc: Vec<String>,
    pub info: Vec<String>,
    #[serde(rename = "languageSource")]
    pub language_source: Vec<serde_json::Value>, // Can be String or Object
    pub gloss: Vec<GlossEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlossEntry {
    pub lang: String,
    pub gender: Option<String>,
    #[serde(rename = "type")]
    pub gloss_type: Option<String>,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct JmdictData {
    pub words: Vec<Entry>,
}
/*
pub fn can_deinflect(entry: &Entry) -> bool {
    entry.sense.iter().any(|sense| {
        sense.part_of_speech.iter().any(|pos| DEINFLECTABLE_POS.contains(&pos.as_str()))
    })
}

pub fn get_base_form(entry: &Entry) -> Option<&str> {
    entry.kanji.first().map(|k| k.text.as_str()) // Get first kanji writing
}
*/
