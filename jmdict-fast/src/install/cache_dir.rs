//! Cache-directory resolution for `Dict::install*`.
//!
//! Three-tier priority chain, in order:
//! 1. `InstallOptions::cache_dir(...)` — caller-scoped, beats everything.
//!    (Resolved by the caller in `install_with`; this module never sees it.)
//! 2. [`init_sdk_cache_dir`] — process-global, first-set-wins via `OnceLock`.
//!    Set once at host startup (Flutter / native shell / a CLI flag).
//! 3. [`platform_default`] — `dirs::cache_dir()` on desktop, hard error on
//!    sandboxed platforms (iOS / Android / WASM) where guessing would crash
//!    at runtime.
//!
//! The OnceLock approach (first-set-wins, no locks on the read path) is
//! copied from the xybrid SDK design — same justification: a process can
//! only sensibly have one cache root, and resetting it mid-run breaks any
//! file handles already mmap'd from the old location.

use crate::error::JmdictError;
use std::path::PathBuf;
use std::sync::OnceLock;

static OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

/// Register a process-global cache directory for `Dict::install*`. First
/// call wins; subsequent calls return [`JmdictError::CacheDirAlreadySet`]
/// rather than silently overwriting (which would leave previously-loaded
/// `Dict`s pointing at a different root than future installs).
///
/// On sandboxed platforms (iOS / Android / WASM) this is **mandatory** —
/// the host gets the right path from a platform API (Flutter's
/// `path_provider`, Android's `Context.getCacheDir`, iOS `FileManager`) and
/// passes it in once at startup.
pub fn init_sdk_cache_dir(path: PathBuf) -> Result<(), JmdictError> {
    OVERRIDE
        .set(path)
        .map_err(|_| JmdictError::CacheDirAlreadySet)
}

/// Resolve the cache root using tiers 2 and 3 of the priority chain. Tier 1
/// (per-call override) is handled by the caller before this is invoked.
pub fn resolved_cache_dir() -> Result<PathBuf, JmdictError> {
    if let Some(p) = OVERRIDE.get() {
        return Ok(p.clone());
    }
    platform_default()
}

/// `dirs::cache_dir()` on platforms where it returns a writable directory
/// for native code; a typed error on platforms where the value would be
/// wrong (iOS) or absent (Android, WASM).
///
/// Listed explicitly per target_os so adding a new platform forces a
/// conscious decision instead of silently falling into a default branch.
fn platform_default() -> Result<PathBuf, JmdictError> {
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    {
        dirs::cache_dir().ok_or(JmdictError::CacheDirRequired {
            platform: std::env::consts::OS,
        })
    }

    #[cfg(target_os = "ios")]
    {
        // `dirs::cache_dir()` on iOS returns `$HOME/Library/Caches`, but
        // `$HOME` inside an iOS app sandbox is set to the app's container
        // by the OS — except when Rust is invoked from a unit test in the
        // simulator, where it can resolve to an unwritable system path.
        // Force the host to inject via path_provider.
        Err(JmdictError::CacheDirRequired { platform: "ios" })
    }

    #[cfg(target_os = "android")]
    {
        // Android apps can't write outside their sandbox; the JVM owns
        // `Context.getCacheDir()` and there's no portable way for native
        // code to reach it. Require an explicit override.
        Err(JmdictError::CacheDirRequired { platform: "android" })
    }

    #[cfg(target_arch = "wasm32")]
    {
        Err(JmdictError::CacheDirRequired { platform: "wasm" })
    }

    #[cfg(not(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "ios",
        target_os = "android",
        target_arch = "wasm32",
    )))]
    {
        Err(JmdictError::CacheDirRequired {
            platform: std::env::consts::OS,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_cache_dir_returns_path_on_desktop() {
        // The OnceLock is process-global; if another test in the same
        // binary already set it, we get that path. Either way the call
        // should succeed on macOS / Linux / Windows builders.
        #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
        {
            let dir = resolved_cache_dir().expect("desktop should have a default cache dir");
            assert!(dir.is_absolute(), "cache dir should be absolute, got {dir:?}");
        }
    }
}
