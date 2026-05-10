//! Caching system for downloading and caching JMdict data files
//!
//! Uses a project-level .cache/ directory for storing downloaded data.

use anyhow::Result;
use std::io::Cursor;
use std::path::PathBuf;

/// Configuration for the caching system
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub cache_file_name: String,
    pub version_file_name: String,
    pub current_version: String,
}

impl CacheConfig {
    /// Create a new cache configuration
    pub fn new(cache_file_name: &str, version_file_name: &str, current_version: &str) -> Self {
        Self {
            cache_file_name: cache_file_name.to_string(),
            version_file_name: version_file_name.to_string(),
            current_version: current_version.to_string(),
        }
    }

    /// Get the cache file paths based on project-level .cache/ directory
    pub fn get_cache_paths(&self) -> Result<(PathBuf, PathBuf)> {
        let cache_dir = get_project_cache_dir()?;
        std::fs::create_dir_all(&cache_dir)?;
        let cache_file = cache_dir.join(&self.cache_file_name);
        let version_file = cache_dir.join(&self.version_file_name);
        Ok((cache_file, version_file))
    }
}

/// Get the project-level .cache/ directory
fn get_project_cache_dir() -> Result<PathBuf> {
    // Walk up from current dir to find Cargo.toml workspace root
    let mut dir = std::env::current_dir()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let contents = std::fs::read_to_string(&cargo_toml)?;
            if contents.contains("[workspace]") {
                return Ok(dir.join(".cache"));
            }
        }
        if !dir.pop() {
            // Fallback to current directory
            return Ok(std::env::current_dir()?.join(".cache"));
        }
    }
}

/// Determines if a download is needed based on cache state.
///
/// When a version mismatch is detected, the stale cache and version files are
/// deleted so a subsequent download failure cannot silently serve wrong-version
/// bytes via the fallback in [`download_and_cache`].
pub fn should_download(config: &CacheConfig) -> Result<bool> {
    let (cache_file, version_file) = config.get_cache_paths()?;

    if cache_file.exists() && version_file.exists() {
        match std::fs::read_to_string(&version_file) {
            Ok(cached_version) => {
                if cached_version.trim() != config.current_version {
                    eprintln!("🔄 Cache version mismatch, removing stale cache...");
                    let _ = std::fs::remove_file(&cache_file);
                    let _ = std::fs::remove_file(&version_file);
                    Ok(true)
                } else {
                    eprintln!("✅ Using cached file at {:?}", cache_file);
                    Ok(false)
                }
            }
            Err(_) => {
                eprintln!("⚠️  Could not read version file, removing cache and re-downloading...");
                let _ = std::fs::remove_file(&cache_file);
                let _ = std::fs::remove_file(&version_file);
                Ok(true)
            }
        }
    } else {
        eprintln!("🌐 No cache found, downloading...");
        Ok(true)
    }
}

/// Downloads and caches data with error handling and fallback
pub fn download_and_cache<F>(config: &CacheConfig, download_fn: F) -> Result<Vec<u8>>
where
    F: FnOnce() -> Result<Vec<u8>>,
{
    let (cache_file, version_file) = config.get_cache_paths()?;

    eprintln!("📥 Downloading to {:?}", cache_file);
    match download_fn() {
        Ok(bytes) => {
            std::fs::write(&cache_file, &bytes)?;
            std::fs::write(&version_file, &config.current_version)?;
            eprintln!("✅ Successfully downloaded and cached");
            Ok(bytes)
        }
        Err(e) => {
            eprintln!("❌ Download failed: {}", e);
            if cache_file.exists() {
                eprintln!("⚠️  Falling back to existing cache (may be stale)");
                Ok(std::fs::read(&cache_file)?)
            } else {
                Err(e)
            }
        }
    }
}

/// Verifies cache integrity and re-downloads if necessary
pub fn verify_and_fix_cache<F>(config: &CacheConfig, download_fn: F) -> Result<Vec<u8>>
where
    F: FnOnce() -> Result<Vec<u8>>,
{
    let (cache_file, _) = config.get_cache_paths()?;

    match std::fs::read(&cache_file) {
        Ok(data) => {
            if data.is_empty() {
                eprintln!("❌ Cache file is empty, re-downloading...");
                download_and_cache(config, download_fn)
            } else {
                Ok(data)
            }
        }
        Err(e) => {
            eprintln!("❌ Could not read cache file: {}, re-downloading...", e);
            download_and_cache(config, download_fn)
        }
    }
}

/// Generic function to load cached data with automatic download and verification
pub fn load_cached_data<F>(config: CacheConfig, download_fn: F) -> Result<Cursor<Vec<u8>>>
where
    F: FnOnce() -> Result<Vec<u8>>,
{
    let bytes = if should_download(&config)? {
        download_and_cache(&config, download_fn)?
    } else {
        verify_and_fix_cache(&config, download_fn)?
    };
    Ok(Cursor::new(bytes))
}
