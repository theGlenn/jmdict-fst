//! Download/extract/load the dictionary data on first use.
//!
//! Enabled via the `install` feature. The core crate stays HTTP-free; turning
//! the feature on adds `ureq` + `tar` + `flate2` + `dirs` and exposes:
//!
//! - [`Dict::install`] / [`Dict::install_with`] — official release tarball.
//! - [`Dict::install_from_url`] — arbitrary `.tar.gz` over HTTPS.
//! - [`Dict::install_from_tarball`] — local `.tar.gz` already on disk.
//! - [`init_sdk_cache_dir`] — host-side override for the cache root.
//!
//! All three install entry points materialize the seven runtime files
//! (`{kana,kanji,romaji,id,gloss}.fst`, `entries.bin`, `gloss_postings.bin`)
//! into `<cache>/jmdict-fast/fmt<N>/<jmdict-version>/` and then call
//! [`Dict::load`] on that directory.

mod cache_dir;
mod options;
mod tarball;

pub use cache_dir::{init_sdk_cache_dir, resolved_cache_dir};
pub use options::{InstallOptions, InstallSource};

use crate::dict::Dict;
use crate::error::JmdictError;
use crate::model::{FORMAT_VERSION, JMDICT_VERSION};
use std::path::{Path, PathBuf};

/// Filenames the loader expects under the install directory. A directory
/// missing any of these is treated as not-yet-installed, even if some files
/// are present (an interrupted extract).
pub(crate) const REQUIRED_FILES: &[&str] = &[
    "entries.bin",
    "kana.fst",
    "kanji.fst",
    "romaji.fst",
    "id.fst",
    "gloss.fst",
    "gloss_postings.bin",
];

/// URL of the canonical release tarball this build was compiled against.
///
/// Pattern: the data tarball is uploaded to the same GitHub release that
/// publishes the crate (`jmdict-fast-v<crate>`), with a filename keyed on the
/// JMdict source version and on-disk format version. Tying it to
/// `CARGO_PKG_VERSION` means a `0.1.3` crate installs the `0.1.3` tarball,
/// so an older crate keeps working against its matching data even after a
/// newer release lands.
pub fn default_data_url() -> String {
    format!(
        "https://github.com/theGlenn/jmdict-fst/releases/download/jmdict-fast-v{crate}/jmdict-data-jmdict{jm}-fmt{fmt}.tar.gz",
        crate = env!("CARGO_PKG_VERSION"),
        jm = JMDICT_VERSION,
        fmt = FORMAT_VERSION,
    )
}

/// Returns the subdirectory inside the resolved cache root where this
/// build's data lives: `jmdict-fast/fmt<N>/<jmdict-version>/`.
///
/// The format-version segment is what makes `Dict::install()` safe across
/// upgrades: a `fmt4` build never sees a `fmt5` build's files, so two
/// crate versions coexisting on one machine don't clobber each other.
pub(crate) fn install_subdir() -> PathBuf {
    PathBuf::from("jmdict-fast")
        .join(format!("fmt{FORMAT_VERSION}"))
        .join(JMDICT_VERSION)
}

/// True iff every file the loader needs is present in `dir`. Used to skip
/// re-extracting on a warm cache and to validate after extraction.
pub(crate) fn install_complete(dir: &Path) -> bool {
    REQUIRED_FILES.iter().all(|f| dir.join(f).exists())
}

impl Dict {
    /// Download the official release tarball into the platform cache and
    /// load it. No-op on the second call (cache hit).
    ///
    /// On iOS/Android/WASM the host must first call [`init_sdk_cache_dir`]
    /// or pass [`InstallOptions::cache_dir`] — see [`JmdictError::CacheDirRequired`].
    pub fn install() -> Result<Self, JmdictError> {
        Self::install_with(InstallOptions::default())
    }

    /// Download an arbitrary tarball URL into the platform cache and load
    /// it. Useful for self-hosted mirrors or pre-release data.
    pub fn install_from_url(url: impl Into<String>) -> Result<Self, JmdictError> {
        Self::install_with(InstallOptions::default().source(InstallSource::Url(url.into())))
    }

    /// Extract a local `.tar.gz` into the platform cache and load it. The
    /// tarball must contain the seven runtime files at the archive root.
    pub fn install_from_tarball(path: impl Into<PathBuf>) -> Result<Self, JmdictError> {
        Self::install_with(InstallOptions::default().source(InstallSource::Tarball(path.into())))
    }

    /// Full install with an explicit options builder — used to override the
    /// cache directory, force a re-extract, or pick a non-default source.
    pub fn install_with(opts: InstallOptions) -> Result<Self, JmdictError> {
        let root = match opts.cache_dir.clone() {
            Some(p) => p,
            None => resolved_cache_dir()?,
        };
        let target = root.join(install_subdir());

        if opts.force || !install_complete(&target) {
            // With force=true the caller wants a clean reinstall —
            // wipe first so files from a prior install that aren't in
            // the new tarball (or stale corrupt versions) don't linger.
            // Skipped on the "first install" path so the OS doesn't see
            // a needless remove/recreate dance.
            if opts.force && target.exists() {
                std::fs::remove_dir_all(&target)?;
            }
            std::fs::create_dir_all(&target)?;
            materialize(&target, &opts.source)?;
            if !install_complete(&target) {
                // Tarball was missing a required file — refuse to load
                // half-installed data rather than producing a confusing
                // "DataCorrupted" later.
                return Err(JmdictError::DataCorrupted);
            }
        }

        Dict::load(&target)
    }
}

/// Drive the source-specific bytes into `target`. Pulled out of
/// `install_with` so the dispatch table is one place.
fn materialize(target: &Path, source: &InstallSource) -> Result<(), JmdictError> {
    match source {
        InstallSource::OfficialRelease => {
            let bytes = tarball::download(&default_data_url())?;
            tarball::extract(&bytes, target)
        }
        InstallSource::Url(url) => {
            let bytes = tarball::download(url)?;
            tarball::extract(&bytes, target)
        }
        InstallSource::Tarball(path) => tarball::extract_from_path(path, target),
    }
}
