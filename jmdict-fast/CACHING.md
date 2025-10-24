# JMdict Caching System

This document explains the caching system used by the JMdict build script and provides troubleshooting commands for maintainers.

## Overview

The JMdict build script (`build.rs`) implements a robust caching system to avoid re-downloading the large JMdict dataset on every build. The system includes version tracking, error handling, and integrity verification.

## How It Works

### Cache Location
- **Build artifacts**: `target/debug/build/jmdict-fast-*/out/`
- **Cache files**: 
  - `jmdict-eng-common.json` - The downloaded JMdict data
  - `jmdict-version.txt` - Version tracking file containing the download URL

### Caching Logic

1. **Version Check**: Compares current download URL with cached version
2. **Smart Download**: Only downloads if:
   - Cache doesn't exist
   - Version mismatch detected
   - Cache file is corrupted or empty
3. **Fallback Strategy**: Uses existing cache if download fails
4. **Integrity Verification**: Validates downloaded data before use

### Status Messages

The build script provides clear feedback with emoji indicators:

- ✅ **Cache Hit**: Using existing cached data
- 🌐 **No Cache**: First-time download
- 🔄 **Version Mismatch**: Updating cache due to URL change
- 📥 **Downloading**: Currently downloading data
- ❌ **Error**: Download or cache operation failed
- ⚠️ **Warning**: Using fallback cache (may be stale)

## Maintenance Commands

### Check Cache Status

```bash
# List all build artifacts
find target/debug/build -name "jmdict*" -type d

# Check cache files
ls -la target/debug/build/jmdict-fast-*/out/

# View cached version
cat target/debug/build/jmdict-fast-*/out/jmdict-version.txt

# Check cache file size
ls -lh target/debug/build/jmdict-fast-*/out/jmdict-eng-common.json
```

### Force Cache Refresh

```bash
# Remove all cache files
rm -rf target/debug/build/jmdict-fast-*/out/jmdict-*

# Clean and rebuild
cargo clean
cargo build

# Remove specific cache files
rm target/debug/build/jmdict-fast-*/out/jmdict-eng-common.json
rm target/debug/build/jmdict-fast-*/out/jmdict-version.txt
```

### Verify Cache Integrity

```bash
# Check if cache files exist and are readable
find target/debug/build -name "jmdict-eng-common.json" -exec file {} \;

# Verify JSON structure (requires jq)
find target/debug/build -name "jmdict-eng-common.json" -exec jq empty {} \; 2>/dev/null && echo "JSON is valid" || echo "JSON is invalid"

# Check file permissions
ls -la target/debug/build/jmdict-fast-*/out/jmdict-*
```

### Debug Build Script

```bash
# Run with verbose output
cargo build --verbose

# Check build script logs
find target/debug/build -name "build-script-build" -exec {} \;

# View build dependencies
cargo tree --workspace
```

### Network Troubleshooting

```bash
# Test download URL accessibility
curl -I "https://github.com/scriptin/jmdict-simplified/releases/download/3.6.1%2B20250714122633/jmdict-eng-3.6.1+20250714122633.json.tgz"

# Check network connectivity
ping github.com

# Verify SSL certificates
openssl s_client -connect github.com:443 -servername github.com
```

## Common Issues and Solutions

### Issue: Cache Not Working
**Symptoms**: Build always downloads data
**Solution**: Check file permissions and disk space
```bash
# Check disk space
df -h target/

# Fix permissions
chmod 644 target/debug/build/jmdict-fast-*/out/jmdict-*
```

### Issue: Corrupted Cache
**Symptoms**: Build fails with JSON parsing errors
**Solution**: Remove corrupted cache and rebuild
```bash
rm target/debug/build/jmdict-fast-*/out/jmdict-eng-common.json
cargo build
```

### Issue: Version Mismatch
**Symptoms**: Frequent cache updates
**Solution**: Check if URL has changed in `build.rs`
```bash
grep "ARTIFACT_URL" build.rs
```

### Issue: Network Errors
**Symptoms**: Download failures
**Solution**: Check network connectivity and try again
```bash
# Clear DNS cache (macOS)
sudo dscacheutil -flushcache

# Retry with different network
cargo clean && cargo build
```

## Configuration

### Environment Variables

- `OUT_DIR`: Build output directory (set by Cargo)
- `JMDICT_DATA`: Alternative data directory (optional)

### Build Script Constants

- `CACHE_FILE_PATH`: Cache file name
- `CACHE_VERSION_FILE`: Version tracking file name
- `ARTIFACT_URL`: Download URL for JMdict data

## Best Practices

1. **Never commit cache files** to version control
2. **Test cache invalidation** when updating URLs
3. **Monitor cache size** to prevent disk space issues
4. **Use CI/CD cache** for faster builds in automated environments
5. **Document URL changes** in commit messages

## Troubleshooting Checklist

- [ ] Cache files exist in expected location
- [ ] File permissions are correct (644)
- [ ] Version file contains current URL
- [ ] JSON file is valid and not empty
- [ ] Network connectivity is working
- [ ] Disk space is sufficient
- [ ] Build script has proper error handling

## Related Files

- `build.rs` - Main build script with caching logic
- `Cargo.toml` - Project dependencies
- `src/lib.rs` - Library code that uses cached data

## Support

For issues with the caching system:

1. Check this document first
2. Run the troubleshooting commands above
3. Check the build script output for error messages
4. Verify network connectivity and disk space
5. Create an issue with detailed error information

---

*Last updated: $(date)* 