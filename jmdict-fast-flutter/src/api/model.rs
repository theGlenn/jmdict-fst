use jmdict_fast_ffi as facade;

// ---------------------------------------------------------------------------
// Records — these become Dart classes with `final` fields.
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct DataVersion {
    pub format_version: u32,
    pub jmdict_version: String,
    pub generated_at: String,
}

impl From<facade::DataVersion> for DataVersion {
    fn from(v: facade::DataVersion) -> Self {
        Self {
            format_version: v.format_version,
            jmdict_version: v.jmdict_version,
            generated_at: v.generated_at,
        }
    }
}

#[derive(Clone)]
pub struct KanjiEntry {
    pub common: bool,
    pub text: String,
    pub tags: Vec<String>,
}

impl From<facade::KanjiEntry> for KanjiEntry {
    fn from(k: facade::KanjiEntry) -> Self {
        Self {
            common: k.common,
            text: k.text,
            tags: k.tags,
        }
    }
}

#[derive(Clone)]
pub struct KanaEntry {
    pub common: bool,
    pub text: String,
    pub tags: Vec<String>,
    pub applies_to_kanji: Vec<String>,
}

impl From<facade::KanaEntry> for KanaEntry {
    fn from(k: facade::KanaEntry) -> Self {
        Self {
            common: k.common,
            text: k.text,
            tags: k.tags,
            applies_to_kanji: k.applies_to_kanji,
        }
    }
}

#[derive(Clone)]
pub struct Xref {
    pub term: String,
    pub reading: Option<String>,
    pub sense_index: Option<u32>,
}

impl From<facade::Xref> for Xref {
    fn from(x: facade::Xref) -> Self {
        Self {
            term: x.term,
            reading: x.reading,
            sense_index: x.sense_index,
        }
    }
}

impl From<Xref> for facade::Xref {
    fn from(x: Xref) -> Self {
        Self {
            term: x.term,
            reading: x.reading,
            sense_index: x.sense_index,
        }
    }
}

#[derive(Clone)]
pub struct LanguageSource {
    pub lang: String,
    pub full: bool,
    pub wasei: bool,
    pub text: Option<String>,
}

impl From<facade::LanguageSource> for LanguageSource {
    fn from(l: facade::LanguageSource) -> Self {
        Self {
            lang: l.lang,
            full: l.full,
            wasei: l.wasei,
            text: l.text,
        }
    }
}

#[derive(Clone)]
pub struct GlossEntry {
    pub lang: String,
    pub gender: Option<String>,
    pub gloss_type: Option<String>,
    pub text: String,
}

impl From<facade::GlossEntry> for GlossEntry {
    fn from(g: facade::GlossEntry) -> Self {
        Self {
            lang: g.lang,
            gender: g.gender,
            gloss_type: g.gloss_type,
            text: g.text,
        }
    }
}

#[derive(Clone)]
pub struct SenseEntry {
    pub part_of_speech: Vec<String>,
    pub applies_to_kanji: Vec<String>,
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

impl From<facade::SenseEntry> for SenseEntry {
    fn from(s: facade::SenseEntry) -> Self {
        Self {
            part_of_speech: s.part_of_speech,
            applies_to_kanji: s.applies_to_kanji,
            applies_to_kana: s.applies_to_kana,
            related: s.related.into_iter().map(Into::into).collect(),
            antonym: s.antonym.into_iter().map(Into::into).collect(),
            field: s.field,
            dialect: s.dialect,
            misc: s.misc,
            info: s.info,
            language_source: s.language_source.into_iter().map(Into::into).collect(),
            gloss: s.gloss.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone)]
pub struct Entry {
    pub id: String,
    pub kanji: Vec<KanjiEntry>,
    pub kana: Vec<KanaEntry>,
    pub sense: Vec<SenseEntry>,
}

impl From<facade::Entry> for Entry {
    fn from(e: facade::Entry) -> Self {
        Self {
            id: e.id,
            kanji: e.kanji.into_iter().map(Into::into).collect(),
            kana: e.kana.into_iter().map(Into::into).collect(),
            sense: e.sense.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone)]
pub enum MatchType {
    Exact,
    Prefix,
    Deinflected,
    Fuzzy,
    Gloss,
}

impl From<facade::MatchType> for MatchType {
    fn from(m: facade::MatchType) -> Self {
        match m {
            facade::MatchType::Exact => MatchType::Exact,
            facade::MatchType::Prefix => MatchType::Prefix,
            facade::MatchType::Deinflected => MatchType::Deinflected,
            facade::MatchType::Fuzzy => MatchType::Fuzzy,
            facade::MatchType::Gloss => MatchType::Gloss,
        }
    }
}

#[derive(Clone)]
pub enum MatchMode {
    Exact,
    Prefix,
    Deinflect,
    Fuzzy,
}

impl From<MatchMode> for facade::MatchMode {
    fn from(m: MatchMode) -> Self {
        match m {
            MatchMode::Exact => facade::MatchMode::Exact,
            MatchMode::Prefix => facade::MatchMode::Prefix,
            MatchMode::Deinflect => facade::MatchMode::Deinflect,
            MatchMode::Fuzzy => facade::MatchMode::Fuzzy,
        }
    }
}

#[derive(Clone)]
pub struct DeinflectionInfo {
    pub original_form: String,
    pub base_form: String,
    pub rules: Vec<String>,
}

impl From<facade::DeinflectionInfo> for DeinflectionInfo {
    fn from(d: facade::DeinflectionInfo) -> Self {
        Self {
            original_form: d.original_form,
            base_form: d.base_form,
            rules: d.rules,
        }
    }
}

#[derive(Clone)]
pub struct LookupResult {
    pub entry: Entry,
    pub match_type: MatchType,
    pub match_key: String,
    pub score: f64,
    pub deinflection: Option<DeinflectionInfo>,
}

impl From<facade::LookupResult> for LookupResult {
    fn from(r: facade::LookupResult) -> Self {
        Self {
            entry: r.entry.into(),
            match_type: r.match_type.into(),
            match_key: r.match_key,
            score: r.score,
            deinflection: r.deinflection.map(Into::into),
        }
    }
}

#[derive(Clone)]
pub struct QueryOptions {
    pub mode: MatchMode,
    pub common_only: bool,
    pub pos: Vec<String>,
    pub misc: Vec<String>,
    pub field: Vec<String>,
    pub dialect: Vec<String>,
    pub limit: Option<u32>,
    pub max_distance: u32,
}

impl From<QueryOptions> for facade::QueryOptions {
    fn from(q: QueryOptions) -> Self {
        Self {
            mode: q.mode.into(),
            common_only: q.common_only,
            pos: q.pos,
            misc: q.misc,
            field: q.field,
            dialect: q.dialect,
            limit: q.limit,
            max_distance: q.max_distance,
        }
    }
}

#[derive(Clone)]
pub struct BatchResult {
    pub term: String,
    pub results: Vec<LookupResult>,
}

impl From<facade::BatchResult> for BatchResult {
    fn from(b: facade::BatchResult) -> Self {
        Self {
            term: b.term,
            results: b.results.into_iter().map(Into::into).collect(),
        }
    }
}
