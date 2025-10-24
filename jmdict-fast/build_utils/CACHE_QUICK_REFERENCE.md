# Cache Quick Reference

## Quick Commands

### Check Cache Status
```bash
ls -la target/debug/build/jmdict-fast-*/out/jmdict-*
cat target/debug/build/jmdict-fast-*/out/jmdict-version.txt
```

### Force Refresh Cache
```bash
rm -rf target/debug/build/jmdict-fast-*/out/jmdict-*
cargo build
```

### Verify Cache Integrity
```bash
find target/debug/build -name "jmdict-eng-common.json" -exec jq empty {} \; 2>/dev/null && echo "✅ Valid" || echo "❌ Invalid"
```

### Debug Build Issues
```bash
cargo clean && cargo build --verbose
```

## Status Indicators

- ✅ **Cache Hit** - Using cached data
- 🌐 **No Cache** - First download
- 🔄 **Version Mismatch** - Updating cache
- 📥 **Downloading** - In progress
- ❌ **Error** - Operation failed
- ⚠️ **Warning** - Using fallback

## Common Issues

| Issue | Command |
|-------|---------|
| Cache not working | `chmod 644 target/debug/build/jmdict-fast-*/out/jmdict-*` |
| Corrupted cache | `rm target/debug/build/jmdict-fast-*/out/jmdict-eng-common.json && cargo build` |
| Network errors | `curl -I "https://github.com/scriptin/jmdict-simplified/releases/download/3.6.1%2B20250714122633/jmdict-eng-3.6.1+20250714122633.json.tgz"` |
| Disk space | `df -h target/` |

## File Locations

- **Cache**: `target/debug/build/jmdict-fast-*/out/`
- **Data**: `jmdict-eng-common.json`
- **Version**: `jmdict-version.txt`
- **Build Script**: `build.rs`

---

📖 **Full Documentation**: See [CACHING.md](./CACHING.md) for detailed information. 