//! Dart-facing install surface.
//!
//! Mirrors `jmdict_fast_ffi::{InstallOptions, InstallSource}` as plain
//! Rust structs so FRB scans them into Dart classes with `final` fields.
//! `init_sdk_cache_dir` is a free function the Dart side calls once at
//! startup with a path obtained from `path_provider`.

use jmdict_fast_ffi as facade;

use super::error::Error;

/// Where the install bytes come from. Mirror of `facade::InstallSource`
/// so FRB describes it in Dart.
#[derive(Debug, Clone)]
pub enum InstallSource {
    /// The GitHub release tarball for this crate / JMdict / format version.
    OfficialRelease,
    /// Any `.tar.gz` reachable over HTTPS.
    Url { url: String },
    /// A `.tar.gz` already on the local filesystem.
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

/// Options record for `Dict.installWith(...)`. POD; all fields optional.
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Cache root for this call. Wins over `init_sdk_cache_dir` and the
    /// platform default.
    pub cache_dir: Option<String>,
    pub source: InstallSource,
    /// Re-extract even when the cache appears complete.
    pub force: bool,
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

/// Register the process-global cache directory for `Dict.install*`. On
/// Flutter this is mandatory before any install call — the host obtains
/// a writable path via `path_provider.getApplicationSupportDirectory()`
/// (or platform-specific equivalent) and passes it in once.
///
/// First call wins; subsequent calls return [`Error::CacheDirAlreadySet`].
pub fn init_sdk_cache_dir(path: String) -> Result<(), Error> {
    facade::init_sdk_cache_dir(path).map_err(Into::into)
}
