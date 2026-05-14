//! FFI-agnostic facade over `jmdict-fast`.
//!
//! This crate intentionally exposes only the shapes that every popular Rust
//! FFI generator can describe: owned data, concrete enums, `Arc<Self>` handles,
//! no lifetimes, no generics, no iterators across the boundary. The generator
//! crates (`jmdict-fast-uniffi`, `jmdict-fast-frb`, `jmdict-fast-bolt`, …)
//! describe these types externally (UDL, FRB scan, bolt IDL) and add their
//! own scaffolding — they should not need to reach into `jmdict-fast` itself.
//!
//! ## Differences from `jmdict-fast`
//!
//! - `Dict::load` / `load_default` return `Arc<Self>` (every generator requires
//!   `Arc` for object handles).
//! - The chainable `QueryBuilder` collapses into a `QueryOptions` record so
//!   callers in foreign languages don't need fluent chaining.
//! - `JmdictError::IoError(std::io::Error)` collapses into `Error::Io { message }`
//!   so the error survives codegen — `io::Error` is not FFI-safe.
//! - Iterators (`iter_entries`) become an explicit `(start, count)` pagination.
//!
//! ## Re-exports
//!
//! Data types (`Entry`, `KanjiEntry`, `Xref`, …) are re-exported unchanged from
//! `jmdict-fast`. They are already plain POD with public fields and no
//! lifetimes — every FFI generator can describe them as records.

use std::sync::Arc;

use jmdict_fast as core;

pub use core::{
    DataVersion, DeinflectionInfo, Entry, GlossEntry, KanaEntry, KanjiEntry, LanguageSource,
    MatchMode, MatchType, SenseEntry, Xref,
};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors surfaced across the FFI boundary.
///
/// `Error::Io` carries the underlying `io::Error` message as a `String` —
/// `std::io::Error` itself is not FFI-safe, so we collapse the variant here.
#[derive(Debug, Clone)]
pub enum Error {
    /// Data files not found at the expected path.
    DataNotFound,
    /// Binary format version mismatch between data and library.
    DataVersionMismatch { expected: u32, found: u32 },
    /// Data files are corrupted or have an invalid format.
    DataCorrupted,
    /// The query was invalid (e.g., empty string).
    InvalidQuery,
    /// An I/O error occurred while reading data files.
    Io { message: String },
    /// Failed to deserialize entry data.
    Deserialization,
    /// `Dict::install*` needs a cache directory on this platform (iOS,
    /// Android, WASM) and the host hasn't registered one via
    /// `init_sdk_cache_dir` / `InstallOptions::cache_dir`.
    ///
    /// `platform` is the target_os string ("ios", "android", "wasm") so
    /// foreign-language callers can branch on it without parsing the
    /// Display message. `&'static str` → `String` for FFI safety.
    #[cfg(feature = "install")]
    CacheDirRequired { platform: String },
    /// `init_sdk_cache_dir` was called twice. The cache root is process-
    /// global and first-set-wins; subsequent calls are rejected rather
    /// than silently leaving older `Dict`s pointing at a stale root.
    #[cfg(feature = "install")]
    CacheDirAlreadySet,
    /// A network request inside `Dict::install*` failed (timeout, DNS,
    /// non-2xx status, oversize body, …).
    #[cfg(feature = "install")]
    Network { message: String },
}

impl Error {
    /// Stable numeric code for each variant. Matches `JmdictError::code()`.
    pub fn code(&self) -> u32 {
        match self {
            Error::DataNotFound => 1,
            Error::DataVersionMismatch { .. } => 2,
            Error::DataCorrupted => 3,
            Error::InvalidQuery => 4,
            Error::Io { .. } => 5,
            Error::Deserialization => 6,
            #[cfg(feature = "install")]
            Error::CacheDirRequired { .. } => 7,
            #[cfg(feature = "install")]
            Error::Network { .. } => 8,
            #[cfg(feature = "install")]
            Error::CacheDirAlreadySet => 9,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DataNotFound => write!(
                f,
                "Dictionary data files not found. Provide a path to load(), set JMDICT_DATA, or place files under dist/."
            ),
            Error::DataVersionMismatch { expected, found } => write!(
                f,
                "Data format version {found}, library expects {expected}. Regenerate with `cargo xtask generate`."
            ),
            Error::DataCorrupted => write!(f, "Dictionary data is corrupted or has an invalid format."),
            Error::InvalidQuery => write!(f, "The search query is invalid."),
            Error::Io { message } => write!(f, "I/O error: {message}"),
            Error::Deserialization => write!(f, "Failed to deserialize dictionary entry data."),
            #[cfg(feature = "install")]
            Error::CacheDirRequired { platform } => write!(
                f,
                "Cache directory required on {platform}: call init_sdk_cache_dir(path) from the host \
                 (e.g. path_provider on Flutter, FileManager on iOS, Context.getCacheDir on Android), \
                 or pass InstallOptions::cache_dir(path) per call."
            ),
            #[cfg(feature = "install")]
            Error::CacheDirAlreadySet => write!(
                f,
                "init_sdk_cache_dir was already called for this process; the cache root is one-shot."
            ),
            #[cfg(feature = "install")]
            Error::Network { message } => write!(f, "Network error during install: {message}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<core::JmdictError> for Error {
    fn from(err: core::JmdictError) -> Self {
        match err {
            core::JmdictError::DataNotFound => Error::DataNotFound,
            core::JmdictError::DataVersionMismatch { expected, found } => {
                Error::DataVersionMismatch { expected, found }
            }
            core::JmdictError::DataCorrupted => Error::DataCorrupted,
            core::JmdictError::InvalidQuery => Error::InvalidQuery,
            core::JmdictError::IoError(e) => Error::Io { message: e.to_string() },
            core::JmdictError::DeserializationError => Error::Deserialization,
            #[cfg(feature = "install")]
            core::JmdictError::CacheDirRequired { platform } => Error::CacheDirRequired {
                platform: platform.to_string(),
            },
            #[cfg(feature = "install")]
            core::JmdictError::CacheDirAlreadySet => Error::CacheDirAlreadySet,
            #[cfg(feature = "install")]
            core::JmdictError::NetworkError(message) => Error::Network { message },
        }
    }
}

// ---------------------------------------------------------------------------
// LookupResult, QueryOptions, BatchResult
// ---------------------------------------------------------------------------

/// A single dictionary lookup hit.
///
/// Mirrors `core::LookupResult` but lives in the FFI surface so FFI codegen
/// can describe it without reaching across crate boundaries.
#[derive(Debug, Clone)]
pub struct LookupResult {
    pub entry: Entry,
    pub match_type: MatchType,
    pub match_key: String,
    pub score: f64,
    pub deinflection: Option<DeinflectionInfo>,
}

impl From<core::LookupResult> for LookupResult {
    fn from(r: core::LookupResult) -> Self {
        Self {
            entry: r.entry,
            match_type: r.match_type,
            match_key: r.match_key,
            score: r.score,
            deinflection: r.deinflection,
        }
    }
}

/// Options for [`Dict::lookup_with_options`] / [`Dict::lookup_batch`].
///
/// Foreign-language callers populate this record instead of chaining
/// `QueryBuilder` setters. All fields have sensible defaults; see
/// [`QueryOptions::default`].
#[derive(Debug, Clone)]
pub struct QueryOptions {
    pub mode: MatchMode,
    pub common_only: bool,
    pub pos: Vec<String>,
    pub misc: Vec<String>,
    pub field: Vec<String>,
    pub dialect: Vec<String>,
    /// `None` means no cap. `u32` (not `usize`) so the type is portable across
    /// 32-/64-bit FFI targets.
    pub limit: Option<u32>,
    /// Only consulted when `mode == MatchMode::Fuzzy`. Clamped to
    /// `core::MAX_FUZZY_DISTANCE` by the underlying builder.
    pub max_distance: u32,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            mode: MatchMode::Exact,
            common_only: false,
            pos: Vec::new(),
            misc: Vec::new(),
            field: Vec::new(),
            dialect: Vec::new(),
            limit: None,
            max_distance: 2,
        }
    }
}

/// Per-term batch result, paired with the original term so callers can
/// correlate output without relying on input order.
#[derive(Debug, Clone)]
pub struct BatchResult {
    pub term: String,
    pub results: Vec<LookupResult>,
}

// ---------------------------------------------------------------------------
// Dict handle
// ---------------------------------------------------------------------------

/// FFI-friendly handle around a loaded dictionary. Construct via
/// [`Dict::load`] or [`Dict::load_default`]; the resulting `Arc<Dict>` can be
/// shared freely across threads and FFI calls.
pub struct Dict {
    inner: core::Dict,
}

impl Dict {
    /// Load all FSTs and entries via real `mmap`.
    pub fn load(path: String) -> Result<Arc<Self>, Error> {
        let inner = core::Dict::load(&path)?;
        Ok(Arc::new(Self { inner }))
    }

    /// Try the same cascade as `core::Dict::load_default` (embedded feature,
    /// `JMDICT_DATA` env var, `dist/`).
    pub fn load_default() -> Result<Arc<Self>, Error> {
        let inner = core::Dict::load_default()?;
        Ok(Arc::new(Self { inner }))
    }

    pub fn entry_count(&self) -> u64 {
        self.inner.entry_count() as u64
    }

    pub fn version(&self) -> DataVersion {
        self.inner.version()
    }

    // ------- Convenience lookups --------------------------------------------

    pub fn lookup_exact(&self, term: String) -> Vec<LookupResult> {
        self.inner
            .lookup_exact(&term)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_partial(&self, prefix: String) -> Vec<LookupResult> {
        self.inner
            .lookup_partial(&prefix)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_exact_with_deinflection(&self, term: String) -> Vec<LookupResult> {
        self.inner
            .lookup_exact_with_deinflection(&term)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_gloss(&self, query: String) -> Vec<LookupResult> {
        self.inner
            .lookup_gloss(&query)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn lookup_by_id(&self, jmdict_id: String) -> Option<LookupResult> {
        self.inner.lookup_by_id(&jmdict_id).map(Into::into)
    }

    pub fn resolve_xref(&self, xref: Xref) -> Vec<LookupResult> {
        self.inner
            .resolve_xref(&xref)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    // ------- Builder-equivalent ---------------------------------------------

    /// Run a single query with full options.
    pub fn lookup_with_options(
        &self,
        term: String,
        options: QueryOptions,
    ) -> Result<Vec<LookupResult>, Error> {
        let pos: Vec<&str> = options.pos.iter().map(String::as_str).collect();
        let misc: Vec<&str> = options.misc.iter().map(String::as_str).collect();
        let field: Vec<&str> = options.field.iter().map(String::as_str).collect();
        let dialect: Vec<&str> = options.dialect.iter().map(String::as_str).collect();

        let mut builder = self
            .inner
            .lookup(&term)
            .mode(options.mode)
            .common_only(options.common_only)
            .pos(&pos)
            .misc(&misc)
            .field(&field)
            .dialect(&dialect)
            .max_distance(options.max_distance);
        if let Some(limit) = options.limit {
            builder = builder.limit(limit as usize);
        }

        Ok(builder.execute()?.into_iter().map(Into::into).collect())
    }

    /// Run the same query options across many terms. Each `BatchResult`
    /// carries its term alongside the hits so callers can correlate output.
    ///
    /// Routes through `core::BatchQueryBuilder` so filter slices are prepared
    /// once and the options record is not cloned per term.
    pub fn lookup_batch(
        &self,
        terms: Vec<String>,
        options: QueryOptions,
    ) -> Result<Vec<BatchResult>, Error> {
        let term_refs: Vec<&str> = terms.iter().map(String::as_str).collect();
        let pos: Vec<&str> = options.pos.iter().map(String::as_str).collect();
        let misc: Vec<&str> = options.misc.iter().map(String::as_str).collect();
        let field: Vec<&str> = options.field.iter().map(String::as_str).collect();
        let dialect: Vec<&str> = options.dialect.iter().map(String::as_str).collect();

        let mut builder = self
            .inner
            .lookup_batch(&term_refs)
            .mode(options.mode)
            .common_only(options.common_only)
            .pos(&pos)
            .misc(&misc)
            .field(&field)
            .dialect(&dialect)
            .max_distance(options.max_distance);
        if let Some(limit) = options.limit {
            builder = builder.limit(limit as usize);
        }

        Ok(builder
            .execute()?
            .into_iter()
            .map(|(term, results)| BatchResult {
                term,
                results: results.into_iter().map(Into::into).collect(),
            })
            .collect())
    }

    // ------- Browsing --------------------------------------------------------

    /// Fetch a single entry by sequential index.
    pub fn get(&self, seq_id: u64) -> Option<Entry> {
        self.inner.get(seq_id)
    }

    /// Paginate over the entry list. `start` is the first sequential index to
    /// return; `count` is the maximum number of entries. Returns fewer than
    /// `count` items at the end of the dictionary. Iterators don't translate
    /// across FFI, so callers loop on `(start, count)` themselves.
    pub fn iter_entries(&self, start: u64, count: u64) -> Vec<Entry> {
        let total = self.inner.entry_count() as u64;
        let end = start.saturating_add(count).min(total);
        (start..end).filter_map(|i| self.inner.get(i)).collect()
    }
}

// ---------------------------------------------------------------------------
// Install surface (feature = "install")
// ---------------------------------------------------------------------------

/// Source of the install bytes. Mirrors `core::install::InstallSource` but
/// lives here so FFI generators can describe it without crossing crate
/// boundaries. Strings (not `PathBuf`) keep the type FFI-safe.
#[cfg(feature = "install")]
#[derive(Debug, Clone)]
pub enum InstallSource {
    /// The GitHub release tarball matching this build's crate / JMdict /
    /// format versions.
    OfficialRelease,
    /// Any `.tar.gz` reachable over HTTPS.
    Url { url: String },
    /// A `.tar.gz` already on the local filesystem (path as string).
    Tarball { path: String },
}

#[cfg(feature = "install")]
impl Default for InstallSource {
    fn default() -> Self {
        InstallSource::OfficialRelease
    }
}

/// Options for [`Dict::install_with`]. POD record, all fields optional —
/// FFI callers populate it instead of chaining builder setters.
#[cfg(feature = "install")]
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Per-call cache directory. Wins over `init_sdk_cache_dir` and the
    /// platform default. `None` falls back to the resolver chain.
    pub cache_dir: Option<String>,
    pub source: InstallSource,
    /// Re-extract even when the cache appears complete (recovers from a
    /// stale or partially-corrupted install).
    pub force: bool,
}

#[cfg(feature = "install")]
impl Dict {
    /// Download the official release tarball into the platform cache and
    /// load it. No-op on a warm cache.
    pub fn install() -> Result<Arc<Self>, Error> {
        Self::install_with(InstallOptions::default())
    }

    /// Download an arbitrary tarball URL and load it.
    pub fn install_from_url(url: String) -> Result<Arc<Self>, Error> {
        Self::install_with(InstallOptions {
            source: InstallSource::Url { url },
            ..Default::default()
        })
    }

    /// Extract a local `.tar.gz` and load it.
    pub fn install_from_tarball(path: String) -> Result<Arc<Self>, Error> {
        Self::install_with(InstallOptions {
            source: InstallSource::Tarball { path },
            ..Default::default()
        })
    }

    /// Full install with explicit options. Returns the loaded `Arc<Dict>`.
    pub fn install_with(options: InstallOptions) -> Result<Arc<Self>, Error> {
        let mut core_opts = core::install::InstallOptions::default()
            .source(match options.source {
                InstallSource::OfficialRelease => core::install::InstallSource::OfficialRelease,
                InstallSource::Url { url } => core::install::InstallSource::Url(url),
                InstallSource::Tarball { path } => {
                    core::install::InstallSource::Tarball(std::path::PathBuf::from(path))
                }
            })
            .force(options.force);
        if let Some(p) = options.cache_dir {
            core_opts = core_opts.cache_dir(std::path::PathBuf::from(p));
        }
        let inner = core::Dict::install_with(core_opts)?;
        Ok(Arc::new(Self { inner }))
    }
}

/// Register a process-global cache directory for `Dict::install*`. First
/// call wins; subsequent calls return [`Error::CacheDirAlreadySet`].
///
/// On iOS / Android / WASM this is **mandatory** — the host gets the right
/// path from a platform API (Flutter's `path_provider`,
/// `Context.getCacheDir`, `FileManager`) and registers it at startup.
#[cfg(feature = "install")]
pub fn init_sdk_cache_dir(path: String) -> Result<(), Error> {
    core::install::init_sdk_cache_dir(std::path::PathBuf::from(path)).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_match_core() {
        // The FFI codes must agree with core so consumers can switch on a
        // shared numeric protocol.
        assert_eq!(Error::DataNotFound.code(), 1);
        assert_eq!(
            Error::DataVersionMismatch {
                expected: 4,
                found: 3
            }
            .code(),
            2
        );
        assert_eq!(Error::DataCorrupted.code(), 3);
        assert_eq!(Error::InvalidQuery.code(), 4);
        assert_eq!(
            Error::Io {
                message: "boom".into()
            }
            .code(),
            5
        );
        assert_eq!(Error::Deserialization.code(), 6);
    }

    #[test]
    fn error_from_jmdict_io_collapses_to_string() {
        let io = std::io::Error::new(std::io::ErrorKind::Other, "disk on fire");
        let core_err = core::JmdictError::IoError(io);
        match Error::from(core_err) {
            Error::Io { message } => assert!(message.contains("disk on fire")),
            other => panic!("expected Error::Io, got {other:?}"),
        }
    }

    #[test]
    fn query_options_defaults() {
        let q = QueryOptions::default();
        assert_eq!(q.mode, MatchMode::Exact);
        assert!(!q.common_only);
        assert!(q.pos.is_empty());
        assert!(q.misc.is_empty());
        assert!(q.field.is_empty());
        assert!(q.dialect.is_empty());
        assert_eq!(q.limit, None);
        assert_eq!(q.max_distance, 2);
    }
}
