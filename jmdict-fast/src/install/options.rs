//! `InstallOptions` builder + `InstallSource` enum.

use std::path::PathBuf;

/// Where the install bytes come from.
#[derive(Debug, Clone)]
pub enum InstallSource {
    /// The GitHub release tarball matching this crate's version, JMdict
    /// version, and format version (see `default_data_url`).
    OfficialRelease,
    /// Any `.tar.gz` reachable over HTTPS.
    Url(String),
    /// A `.tar.gz` already on the local filesystem.
    Tarball(PathBuf),
}

impl Default for InstallSource {
    fn default() -> Self {
        InstallSource::OfficialRelease
    }
}

/// Configuration for [`crate::Dict::install_with`]. All fields are optional;
/// the builder methods return `self` so calls chain.
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Overrides both the per-process `init_sdk_cache_dir` and the platform
    /// default. Use this for tests or when the host wants per-install
    /// isolation.
    pub(crate) cache_dir: Option<PathBuf>,
    /// Where the bytes come from. Defaults to [`InstallSource::OfficialRelease`].
    pub(crate) source: InstallSource,
    /// Re-extract even if the target directory already looks complete.
    /// Doesn't redownload if the source is a local tarball.
    pub(crate) force: bool,
}

impl InstallOptions {
    /// Use a specific cache root for this install, ignoring the global
    /// override and the platform default. Bypassing the global is what
    /// makes the test suite able to install into a tmpdir without
    /// poisoning the process for sibling tests.
    pub fn cache_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(path.into());
        self
    }

    /// Pick where the install bytes come from. Defaults to the official
    /// GitHub release tarball.
    pub fn source(mut self, source: InstallSource) -> Self {
        self.source = source;
        self
    }

    /// Re-extract even if every required file is already present. Useful
    /// when the caller knows the cache has gone stale (e.g. a partial
    /// tarball that happened to write all seven files but with wrong
    /// contents — rare, but a deterministic escape hatch).
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }
}
