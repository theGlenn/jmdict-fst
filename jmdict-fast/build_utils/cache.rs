//! Reusable caching system for downloading and caching data files
//!
//! This module provides a generic caching system that can be used to download
//! and cache any type of data with version tracking and integrity verification.

use anyhow::Result;
use std::env;
use std::io::Cursor;
use std::path::{Path, PathBuf};

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

    /// Get the cache file paths based on OUT_DIR environment variable
    pub fn get_cache_paths(&self) -> Result<(PathBuf, PathBuf)> {
        let dir_path = env::var("OUT_DIR")?;
        let cache_file = Path::new(&dir_path).join(&self.cache_file_name);
        let version_file = Path::new(&dir_path).join(&self.version_file_name);
        Ok((cache_file, version_file))
    }
}

/// Determines if a download is needed based on cache state
pub fn should_download(config: &CacheConfig) -> Result<bool> {
    let (cache_file, version_file) = config.get_cache_paths()?;

    if cache_file.exists() && version_file.exists() {
        match std::fs::read_to_string(&version_file) {
            Ok(cached_version) => {
                if cached_version.trim() != config.current_version {
                    eprintln!("🔄 Cache version mismatch, updating...");
                    Ok(true)
                } else {
                    eprintln!("✅ Using cached file at {:?}", cache_file);
                    Ok(false)
                }
            }
            Err(_) => {
                eprintln!("⚠️  Could not read version file, re-downloading...");
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
            // Write the data first, then the version (atomic-like)
            std::fs::write(&cache_file, &bytes)?;
            std::fs::write(&version_file, &config.current_version)?;
            eprintln!("✅ Successfully downloaded and cached");
            Ok(bytes)
        }
        Err(e) => {
            eprintln!("❌ Download failed: {}", e);
            // If download fails but cache exists, try to use it
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
    F: FnOnce() -> Result<Vec<u8>> + Clone,
{
    if should_download(&config)? {
        download_and_cache(&config, download_fn.clone())?;
    }

    let data = verify_and_fix_cache(&config, download_fn)?;
    Ok(Cursor::new(data))
}

/// Utility function to clear cache files
#[allow(dead_code)]
pub fn clear_cache(config: &CacheConfig) -> Result<()> {
    let (cache_file, version_file) = config.get_cache_paths()?;

    if cache_file.exists() {
        std::fs::remove_file(&cache_file)?;
        eprintln!("🗑️  Removed cache file: {:?}", cache_file);
    }

    if version_file.exists() {
        std::fs::remove_file(&version_file)?;
        eprintln!("🗑️  Removed version file: {:?}", version_file);
    }

    Ok(())
}

/// Utility function to check cache status
#[allow(dead_code)]
pub fn check_cache_status(config: &CacheConfig) -> Result<CacheStatus> {
    let (cache_file, version_file) = config.get_cache_paths()?;

    if !cache_file.exists() {
        return Ok(CacheStatus::NotFound);
    }

    if !version_file.exists() {
        return Ok(CacheStatus::NoVersion);
    }

    match std::fs::read_to_string(&version_file) {
        Ok(cached_version) => {
            if cached_version.trim() == config.current_version {
                Ok(CacheStatus::Valid)
            } else {
                Ok(CacheStatus::Outdated)
            }
        }
        Err(_) => Ok(CacheStatus::Corrupted),
    }
}

/// Represents the current status of the cache
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum CacheStatus {
    /// Cache file doesn't exist
    NotFound,
    /// Cache exists but no version file
    NoVersion,
    /// Cache is valid and up to date
    Valid,
    /// Cache exists but version is outdated
    Outdated,
    /// Cache files are corrupted
    Corrupted,
}

impl std::fmt::Display for CacheStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheStatus::NotFound => write!(f, "Cache not found"),
            CacheStatus::NoVersion => write!(f, "Cache exists but no version file"),
            CacheStatus::Valid => write!(f, "Cache is valid"),
            CacheStatus::Outdated => write!(f, "Cache is outdated"),
            CacheStatus::Corrupted => write!(f, "Cache is corrupted"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cache_config_new() {
        let config = CacheConfig::new("test.json", "test.version", "v1.0.0");
        assert_eq!(config.cache_file_name, "test.json");
        assert_eq!(config.version_file_name, "test.version");
        assert_eq!(config.current_version, "v1.0.0");
    }

    #[test]
    fn test_cache_status_display() {
        assert_eq!(CacheStatus::Valid.to_string(), "Cache is valid");
        assert_eq!(CacheStatus::NotFound.to_string(), "Cache not found");
        assert_eq!(CacheStatus::Outdated.to_string(), "Cache is outdated");
    }
}
