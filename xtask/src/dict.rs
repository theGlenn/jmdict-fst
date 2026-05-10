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
pub struct Xref {
    pub term: String,
    pub reading: Option<String>,
    pub sense_index: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LanguageSource {
    pub lang: String,
    pub full: bool,
    pub wasei: bool,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlossEntry {
    pub lang: String,
    pub gender: Option<String>,
    #[serde(rename = "type")]
    pub gloss_type: Option<String>,
    pub text: String,
}

// --- Raw JSON types for deserialization from JMdict simplified JSON ---

#[derive(Debug, Deserialize)]
pub struct JmdictData {
    pub words: Vec<RawEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RawEntry {
    pub id: String,
    pub kanji: Vec<KanjiEntry>,
    pub kana: Vec<KanaEntry>,
    pub sense: Vec<RawSenseEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RawSenseEntry {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: Vec<String>,
    #[serde(rename = "appliesToKanji")]
    pub applies_to_kanji: Vec<String>,
    #[serde(rename = "appliesToKana")]
    pub applies_to_kana: Vec<String>,
    pub related: Vec<Vec<serde_json::Value>>,
    pub antonym: Vec<Vec<serde_json::Value>>,
    pub field: Vec<String>,
    pub dialect: Vec<String>,
    pub misc: Vec<String>,
    pub info: Vec<String>,
    #[serde(rename = "languageSource")]
    pub language_source: Vec<RawLanguageSource>,
    pub gloss: Vec<GlossEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RawLanguageSource {
    pub lang: String,
    pub full: bool,
    pub wasei: bool,
    pub text: Option<String>,
}

fn parse_xref(raw: &[serde_json::Value]) -> Xref {
    let term = raw
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let reading = raw.get(1).and_then(|v| v.as_str()).map(|s| s.to_string());
    let sense_index = raw.get(1).and_then(|v| v.as_u64()).map(|n| n as u32)
        .or_else(|| raw.get(2).and_then(|v| v.as_u64()).map(|n| n as u32));
    Xref {
        term,
        reading,
        sense_index,
    }
}

impl RawEntry {
    pub fn into_entry(self) -> Entry {
        Entry {
            id: self.id,
            kanji: self.kanji,
            kana: self.kana,
            sense: self.sense.into_iter().map(|s| s.into_sense()).collect(),
        }
    }
}

impl RawSenseEntry {
    pub fn into_sense(self) -> SenseEntry {
        SenseEntry {
            part_of_speech: self.part_of_speech,
            applies_to_kanji: self.applies_to_kanji,
            applies_to_kana: self.applies_to_kana,
            related: self.related.iter().map(|r| parse_xref(r)).collect(),
            antonym: self.antonym.iter().map(|a| parse_xref(a)).collect(),
            field: self.field,
            dialect: self.dialect,
            misc: self.misc,
            info: self.info,
            language_source: self
                .language_source
                .into_iter()
                .map(|ls| LanguageSource {
                    lang: ls.lang,
                    full: ls.full,
                    wasei: ls.wasei,
                    text: ls.text,
                })
                .collect(),
            gloss: self.gloss,
        }
    }
}
