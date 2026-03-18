use serde::{Deserialize, Serialize};

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
