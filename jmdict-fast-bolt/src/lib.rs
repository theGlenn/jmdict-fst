//! BoltFFI bindings for [`jmdict-fast`].
//!
//! This crate describes the [`jmdict_fast_ffi`] facade externally for the
//! BoltFFI generator. Following BoltFFI's convention:
//!
//! - Records (POD types) are mirrored as `#[data]` structs in this crate —
//!   proc macros must live on the type definitions, not on re-exports, so we
//!   can't decorate the facade's types directly.
//! - The error enum is marked `#[error]` so it surfaces as a typed exception
//!   in target languages.
//! - The `Dict` class uses `#[export] impl Dict { ... }`; BoltFFI handles the
//!   heap allocation and FFI handle internally — no `Arc<Self>` return is
//!   required (unlike uniffi).
//!
//! Run `boltffi pack all --release` (or per-target, e.g. `boltffi pack apple`)
//! to generate the Swift / Kotlin / Java / C# / WASM bindings from this crate.

use std::sync::Arc;

use boltffi::*;
use jmdict_fast_ffi as facade;

// ---------------------------------------------------------------------------
// Records
// ---------------------------------------------------------------------------

#[data]
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

#[data]
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

#[data]
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

#[data]
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

#[data]
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

#[data]
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

#[data]
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

#[data]
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

#[data]
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

#[data]
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

#[data]
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

#[data]
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

#[data]
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

#[data]
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

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[error]
#[derive(Debug, Clone)]
pub enum Error {
    DataNotFound,
    DataVersionMismatch { expected: u32, found: u32 },
    DataCorrupted,
    InvalidQuery,
    Io { message: String },
    Deserialization,
    CacheDirRequired { platform: String },
    CacheDirAlreadySet,
    Network { message: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Reuse the facade's user-facing strings so all FFI consumers print
        // the same diagnostics.
        let facade_err: facade::Error = self.clone().into();
        std::fmt::Display::fmt(&facade_err, f)
    }
}

impl std::error::Error for Error {}

impl From<facade::Error> for Error {
    fn from(e: facade::Error) -> Self {
        match e {
            facade::Error::DataNotFound => Error::DataNotFound,
            facade::Error::DataVersionMismatch { expected, found } => {
                Error::DataVersionMismatch { expected, found }
            }
            facade::Error::DataCorrupted => Error::DataCorrupted,
            facade::Error::InvalidQuery => Error::InvalidQuery,
            facade::Error::Io { message } => Error::Io { message },
            facade::Error::Deserialization => Error::Deserialization,
            facade::Error::CacheDirRequired { platform } => Error::CacheDirRequired { platform },
            facade::Error::CacheDirAlreadySet => Error::CacheDirAlreadySet,
            facade::Error::Network { message } => Error::Network { message },
        }
    }
}

impl From<Error> for facade::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::DataNotFound => facade::Error::DataNotFound,
            Error::DataVersionMismatch { expected, found } => {
                facade::Error::DataVersionMismatch { expected, found }
            }
            Error::DataCorrupted => facade::Error::DataCorrupted,
            Error::InvalidQuery => facade::Error::InvalidQuery,
            Error::Io { message } => facade::Error::Io { message },
            Error::Deserialization => facade::Error::Deserialization,
            Error::CacheDirRequired { platform } => facade::Error::CacheDirRequired { platform },
            Error::CacheDirAlreadySet => facade::Error::CacheDirAlreadySet,
            Error::Network { message } => facade::Error::Network { message },
        }
    }
}

// ---------------------------------------------------------------------------
// Dict class
// ---------------------------------------------------------------------------

/// FFI-exported dictionary handle. BoltFFI manages the lifetime — target
/// languages receive an opaque handle and call methods through it.
pub struct Dict {
    inner: Arc<facade::Dict>,
}

#[export]
impl Dict {
    /// Load from an explicit data directory.
    pub fn load(path: String) -> Result<Self, Error> {
        let inner = facade::Dict::load(path)?;
        Ok(Self { inner })
    }

    /// Try the default cascade (embedded → `JMDICT_DATA` env → `dist/`).
    pub fn load_default() -> Result<Self, Error> {
        let inner = facade::Dict::load_default()?;
        Ok(Self { inner })
    }

    pub fn entry_count(&self) -> u64 {
        self.inner.entry_count()
    }

    pub fn version(&self) -> DataVersion {
        self.inner.version().into()
    }

    pub fn lookup_exact(&self, term: String) -> Vec<LookupResult> {
        self.inner.lookup_exact(term).into_iter().map(Into::into).collect()
    }

    pub fn lookup_partial(&self, prefix: String) -> Vec<LookupResult> {
        self.inner
            .lookup_partial(prefix)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_exact_with_deinflection(&self, term: String) -> Vec<LookupResult> {
        self.inner
            .lookup_exact_with_deinflection(term)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_gloss(&self, query: String) -> Vec<LookupResult> {
        self.inner
            .lookup_gloss(query)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_by_id(&self, jmdict_id: String) -> Option<LookupResult> {
        self.inner.lookup_by_id(jmdict_id).map(Into::into)
    }

    pub fn lookup_with_options(
        &self,
        term: String,
        options: QueryOptions,
    ) -> Result<Vec<LookupResult>, Error> {
        let results = self.inner.lookup_with_options(term, options.into())?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub fn lookup_batch(
        &self,
        terms: Vec<String>,
        options: QueryOptions,
    ) -> Result<Vec<BatchResult>, Error> {
        let results = self.inner.lookup_batch(terms, options.into())?;
        Ok(results.into_iter().map(Into::into).collect())
    }

    pub fn resolve_xref(&self, xref: Xref) -> Vec<LookupResult> {
        self.inner
            .resolve_xref(xref.into())
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn get(&self, seq_id: u64) -> Option<Entry> {
        self.inner.get(seq_id).map(Into::into)
    }

    pub fn iter_entries(&self, start: u64, count: u64) -> Vec<Entry> {
        self.inner
            .iter_entries(start, count)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    // ----- install -------------------------------------------------------
    // BoltFFI generates a single `dict_free` per `#[export] impl` block, so
    // these methods have to share the block above instead of getting a
    // sibling `#[cfg(feature = "install")] #[export] impl Dict`.

    pub fn install() -> Result<Self, Error> {
        let inner = facade::Dict::install()?;
        Ok(Self { inner })
    }

    pub fn install_from_url(url: String) -> Result<Self, Error> {
        let inner = facade::Dict::install_from_url(url)?;
        Ok(Self { inner })
    }

    pub fn install_from_tarball(path: String) -> Result<Self, Error> {
        let inner = facade::Dict::install_from_tarball(path)?;
        Ok(Self { inner })
    }

    pub fn install_with(options: InstallOptions) -> Result<Self, Error> {
        let inner = facade::Dict::install_with(options.into())?;
        Ok(Self { inner })
    }
}

// ---------------------------------------------------------------------------
// Install surface (feature = "install")
// ---------------------------------------------------------------------------

#[data]
#[derive(Clone)]
pub enum InstallSource {
    OfficialRelease,
    Url { url: String },
    Tarball { path: String },
}

impl Default for InstallSource {
    fn default() -> Self {
        InstallSource::OfficialRelease
    }
}

impl From<InstallSource> for facade::InstallSource {
    fn from(s: InstallSource) -> Self {
        match s {
            InstallSource::OfficialRelease => facade::InstallSource::OfficialRelease,
            InstallSource::Url { url } => facade::InstallSource::Url { url },
            InstallSource::Tarball { path } => facade::InstallSource::Tarball { path },
        }
    }
}

#[data]
#[derive(Clone)]
pub struct InstallOptions {
    pub cache_dir: Option<String>,
    pub source: InstallSource,
    pub force: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            cache_dir: None,
            source: InstallSource::default(),
            force: false,
        }
    }
}

impl From<InstallOptions> for facade::InstallOptions {
    fn from(o: InstallOptions) -> Self {
        facade::InstallOptions {
            cache_dir: o.cache_dir,
            source: o.source.into(),
            force: o.force,
        }
    }
}

/// Register a process-global cache directory for `Dict::install*`. The
/// host must call this once at startup on iOS / Android / WASM with a
/// path obtained from a platform API.
#[export]
pub fn init_sdk_cache_dir(path: String) -> Result<(), Error> {
    facade::init_sdk_cache_dir(path).map_err(Into::into)
}
