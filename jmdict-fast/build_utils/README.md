# Build Utils - Reusable Caching System

This module provides a generic caching system that can be used to download and cache any type of data with version tracking and integrity verification.

## Overview

The caching system is designed to be:
- **Reusable**: Can be used for any type of downloadable data
- **Robust**: Handles network failures, corrupted files, and version mismatches
- **Efficient**: Only downloads when necessary
- **Safe**: Atomic-like write operations and fallback mechanisms

## Quick Start

```rust
use build_utils::cache::{CacheConfig, load_cached_data};

fn main() -> Result<()> {
    let config = CacheConfig::new(
        "my-data.json",           // Cache file name
        "my-data.version",        // Version file name
        "https://example.com/v1.0.0", // Current version/URL
    );
    
    let data = load_cached_data(config, download_my_data)?;
    // Use the cached data...
    Ok(())
}

fn download_my_data() -> Result<Vec<u8>> {
    // Your download logic here
    Ok(vec![/* downloaded bytes */])
}
```

## Core Components

### CacheConfig

Configuration for the caching system:

```rust
pub struct CacheConfig {
    pub cache_file_name: String,    // Name of the cache file
    pub version_file_name: String,  // Name of the version tracking file
    pub current_version: String,    // Current version identifier
}
```

### Main Functions

#### `load_cached_data<F>(config: CacheConfig, download_fn: F) -> Result<Cursor<Vec<u8>>>`

The main function that handles the entire caching workflow:

1. Checks if download is needed
2. Downloads and caches data if necessary
3. Verifies cache integrity
4. Returns the data as a `Cursor<Vec<u8>>`

#### `should_download(config: &CacheConfig) -> Result<bool>`

Determines if a download is needed based on:
- Cache file existence
- Version file existence
- Version comparison

#### `download_and_cache<F>(config: &CacheConfig, download_fn: F) -> Result<Vec<u8>>`

Downloads data and caches it with error handling:
- Downloads using the provided function
- Writes data to cache file
- Writes version to version file
- Falls back to existing cache if download fails

#### `verify_and_fix_cache<F>(config: &CacheConfig, download_fn: F) -> Result<Vec<u8>>`

Verifies cache integrity and re-downloads if necessary:
- Checks if cache file is readable
- Checks if cache file is empty
- Re-downloads if cache is corrupted

## Utility Functions

### `clear_cache(config: &CacheConfig) -> Result<()>`

Removes cache files for a clean slate.

### `check_cache_status(config: &CacheConfig) -> Result<CacheStatus>`

Returns the current status of the cache:

```rust
pub enum CacheStatus {
    NotFound,    // Cache file doesn't exist
    NoVersion,   // Cache exists but no version file
    Valid,       // Cache is valid and up to date
    Outdated,    // Cache exists but version is outdated
    Corrupted,   // Cache files are corrupted
}
```

## Usage Examples

### Basic Usage

```rust
use build_utils::cache::{CacheConfig, load_cached_data};

fn load_my_dataset() -> Result<()> {
    let config = CacheConfig::new(
        "dataset.json",
        "dataset.version", 
        "v2.1.0"
    );
    
    let data = load_cached_data(config, download_dataset)?;
    // Process the data...
    Ok(())
}
```

### Custom Download Function

```rust
use reqwest;

async fn download_dataset() -> Result<Vec<u8>> {
    let response = reqwest::get("https://api.example.com/dataset").await?;
    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}
```

### Cache Management

```rust
use build_utils::cache::{CacheConfig, clear_cache, check_cache_status};

fn manage_cache() -> Result<()> {
    let config = CacheConfig::new("data.json", "data.version", "v1.0.0");
    
    // Check cache status
    match check_cache_status(&config)? {
        CacheStatus::Valid => println!("Cache is up to date"),
        CacheStatus::Outdated => println!("Cache needs updating"),
        CacheStatus::NotFound => println!("No cache found"),
        _ => println!("Cache has issues"),
    }
    
    // Clear cache if needed
    clear_cache(&config)?;
    
    Ok(())
}
```

## Error Handling

The caching system provides robust error handling:

- **Network failures**: Falls back to existing cache if available
- **Corrupted files**: Automatically re-downloads
- **Version mismatches**: Updates cache with new version
- **Permission issues**: Provides clear error messages

## Status Messages

The system provides informative status messages:

- ✅ **Cache Hit** - Using cached data
- 🌐 **No Cache** - First download
- 🔄 **Version Mismatch** - Updating cache
- 📥 **Downloading** - In progress
- ❌ **Error** - Operation failed
- ⚠️ **Warning** - Using fallback
- 🗑️ **Cleared** - Cache removed

## Best Practices

1. **Use meaningful version strings**: Include URLs or version numbers that change when the data changes
2. **Handle errors gracefully**: The system provides fallbacks, but you should handle errors in your download functions
3. **Test cache invalidation**: Ensure your version strings change when data changes
4. **Monitor cache size**: Large cache files can accumulate over time
5. **Use async when possible**: For network operations, consider using async/await

## Integration with Build Scripts

This system is designed to work seamlessly with Cargo build scripts:

```rust
// In build.rs
mod build_utils;
use build_utils::cache::{CacheConfig, load_cached_data};

fn main() -> Result<()> {
    let config = CacheConfig::new(
        "dictionary.json",
        "dictionary.version",
        "https://github.com/user/repo/releases/latest/download/dict.json"
    );
    
    let data = load_cached_data(config, download_dictionary)?;
    // Process and generate build artifacts...
    Ok(())
}
```

## Migration from Direct Downloads

If you're migrating from direct downloads, the pattern is:

**Before:**
```rust
let response = reqwest::get(url).await?;
let data = response.bytes().await?;
```

**After:**
```rust
let config = CacheConfig::new("file.json", "file.version", url);
let data = load_cached_data(config, || reqwest::get(url).await?.bytes().await?.to_vec())?;
```

This provides automatic caching, version tracking, and error handling with minimal code changes. 