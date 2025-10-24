# Preparing for crates.io Publication

## Executive Summary

This document outlines the steps and decisions needed to publish `jmdict-fast` and `bunpo` to crates.io. The most critical issue is **the handling of generated FST and binary files**, which are currently embedded into the crate via `include_bytes!()`.

---

## 🚨 Critical Issue: Binary File Hosting

### Current State

The crate generates ~17MB of binary data (FST indexes + entry blob) during the build process via `build.rs`. These files are then embedded directly into the compiled binary using `include_bytes!()` (see `jmdict-fast/src/lib.rs:115-123`).

**Files generated:**
- `kanji.fst` (~243KB)
- `kana.fst` (~257KB)  
- `romaji.fst` (~388KB)
- `id.fst` (~variable)
- `entries.bin` (~16MB)

**Total:** ~17MB of binary data embedded per version

### The Problem

Embedding 17MB of data into a crate has several implications:

1. **Large crate size**: Every user downloading `jmdict-fast` will download ~17MB+ of embedded data
2. **Long build times**: Users must download JMdict JSON and rebuild FSTs during `cargo build`
3. **Network dependency**: Build requires network access to download source data
4. **Version management**: Updating dictionary requires new crate version
5. **Storage costs**: Each crate version stores full dictionary data

### Options to Consider

#### Option 1: Keep Current Approach (Embedded)

**Pros:**
- ✅ Works immediately
- ✅ No external dependencies
- ✅ Self-contained
- ✅ Offline-first after initial build

**Cons:**
- ❌ Large crate size (~17MB per version)
- ❌ Slow builds (download + process 22K entries)
- ❌ Network required during build
- ❌ Rebuild on every cargo update

**Verdict:** Acceptable for initial release, but not ideal long-term.

#### Option 2: Host Files Separately + Download at Runtime

**Approach:** Host pre-built FST/bin files on GitHub Releases or CDN, download at runtime.

**Pros:**
- ✅ Smaller crate size
- ✅ Faster builds
- ✅ Dictionary updates without crate updates
- ✅ Better caching

**Cons:**
- ❌ Runtime network dependency
- ❌ Need hosting infrastructure
- ❌ More complex error handling
- ❌ Offline support concerns

**Implementation:**
```rust
// Add feature flag: "embedded" (default) vs "download"
#[cfg(feature = "download")]
pub fn load_default() -> Result<Self> {
    let cache_dir = get_cache_dir()?;
    if !files_exist(&cache_dir) {
        download_files_from_release()?;
    }
    Self::load(&cache_dir)
}
```

#### Option 3: Optional Features Strategy

**Approach:** Make embedded data optional via feature flags.

**Features:**
- `embedded` (default): Include dictionary data in binary
- `download`: Download dictionary at runtime
- `build`: Build dictionary from source (for development)

**Pros:**
- ✅ Flexible for different use cases
- ✅ Users choose their preference
- ✅ Can gradually migrate

**Cons:**
- ❌ More complex API
- ❌ Need to maintain multiple paths

#### Option 4: Separate Data Package

**Approach:** Publish dictionary data as separate crate(s).

**Structure:**
- `jmdict-fast` - Core library (lightweight)
- `jmdict-fast-data` - Dictionary data (or multiple versioned crates)

**Pros:**
- ✅ Clean separation of concerns
- ✅ Multiple dictionary versions possible
- ✅ Core library stays small

**Cons:**
- ❌ Need to publish multiple crates
- ❌ More complex dependency management

### Recommendation

**Short-term:** Keep current embedded approach for initial v0.1.1 release.
- It works and is predictable
- Users expect some size for dictionary data
- No infrastructure needed

**Long-term:** Migrate to Option 2 (runtime download) with `embedded` as fallback feature.
- Reduces crate size by 95%+
- Enables dictionary updates without crate updates
- Better user experience

---

## ✅ Required Fixes for crates.io

### 1. LICENSE File (Required)

**Issue:** Cargo.toml specifies `license = "MIT"` but no LICENSE file exists.

**Action:** Create LICENSE file with MIT text.

### 2. Author Format (Required)

**Issue:** Current author is `"GSO"` which is incomplete.

**Required format:** `"Name <email@example.com>"`

**Action:** Update workspace Cargo.toml authors field.

### 3. Repository URL (Recommended)

**Issue:** No repository specified in Cargo.toml.

**Action:** Add:
```toml
repository = "https://github.com/yourusername/jmdict-fst"
```

### 4. Package Metadata (Recommended)

**Missing fields:**
- `description` (required for jmdict-fast, missing for bunpo)
- `keywords` (e.g., "japanese", "dictionary", "jmdict", "lookup")
- `categories` (e.g., "text-processing", "internationalization")

**Action:** Add to both crate Cargo.toml files.

### 5. Documentation Links (Recommended)

Add to Cargo.toml:
```toml
documentation = "https://docs.rs/jmdict-fast"
homepage = "https://github.com/yourusername/jmdict-fst"
```

### 6. Bunpo Crate

`bunpo` appears to be a deinflector library. Decide:
- **Option A:** Publish separately as `bunpo` crate
- **Option B:** Keep internal to jmdict-fast (don't publish)

If publishing separately, it needs:
- Proper description
- Keywords/categories
- License

---

## 📋 Pre-Publication Checklist

### Workspace Setup
- [ ] Create LICENSE file (MIT)
- [ ] Fix author format in workspace Cargo.toml
- [ ] Add repository URL
- [ ] Verify .gitignore excludes target/ properly

### jmdict-fast Crate
- [ ] Add description field
- [ ] Add keywords (japanese, dictionary, jmdict, lookup, fst)
- [ ] Add categories (text-processing, internationalization)
- [ ] Add documentation and homepage URLs
- [ ] Decide on binary file strategy (see above)
- [ ] Test `cargo package` locally
- [ ] Test `cargo publish --dry-run`
- [ ] Ensure all examples work
- [ ] Document any build requirements in README

### bunpo Crate
- [ ] Decide: publish separately or keep internal?
- [ ] If publishing: add description, keywords, categories
- [ ] If keeping internal: ensure dependency is path-based

### Testing
- [ ] Run `cargo test` on both crates
- [ ] Test building with `cargo build --release`
- [ ] Verify benchmarks work
- [ ] Test examples compile and run

### Documentation
- [ ] Update README with installation instructions
- [ ] Document binary file size implications
- [ ] Add badges to README (crates.io version, docs.rs)
- [ ] Ensure all external links work

### Legal/Licensing
- [ ] Confirm MIT license compatibility with JMdict data
- [ ] Document JMdict attribution properly
- [ ] Include all third-party license notices

---

## 🔍 Testing Publication

Before publishing to crates.io:

```bash
# 1. Check package metadata
cargo package --list

# 2. Verify what will be uploaded
cargo package

# 3. Test install locally
cargo install --path .

# 4. Dry run to crates.io
cargo publish --dry-run

# 5. Check crate size
du -sh target/package/jmdict-fast-*
```

---

## 📊 Estimated Timeline

- **Phase 1 (Quick fixes):** 1-2 hours
  - Create LICENSE
  - Fix Cargo.toml metadata
  - Test package
  
- **Phase 2 (Decision & implementation):** 4-8 hours
  - Decide on binary file strategy
  - Implement if changing approach
  - Update documentation
  
- **Phase 3 (Testing):** 2-4 hours
  - Full test suite
  - Verify examples
  - Test publication
  
- **Total:** 1-2 days depending on chosen approach

---

## 🎯 Recommended Publication Strategy

### Initial Release (v0.1.1)

1. **Keep embedded approach** (Option 1)
2. **Publish jmdict-fast only** (keep bunpo internal initially)
3. **Accept ~17MB crate size** for now
4. **Document clearly** the size and build requirements

### Future Release (v0.2.0+)

1. **Add feature flags** for embedded vs download
2. **Implement runtime download** option
3. **Make "download" default** for better UX
4. **Keep "embedded"** as optional feature

This allows:
- Quick initial publication
- Time to implement better solution
- Backward compatibility
- Gradual migration path

---

## 📞 Questions to Address

1. **Do you want to publish bunpo separately?**
   - It's a useful general-purpose deinflector
   - Could benefit other projects
   - Requires separate maintenance

2. **What's your hosting budget/capability?**
   - GitHub Releases: Free, versioned
   - CDN: Better performance, may cost
   - Self-hosted: Full control, maintenance burden

3. **How often will dictionary data update?**
   - Frequent updates favor download approach
   - Infrequent updates favor embedded approach

4. **Who's the primary user?**
   - CLI tools: Embedded works fine
   - Web services: Download better
   - Mobile apps: Embedded better

---

## 🚀 Next Steps

1. Review this document
2. Make decision on binary file strategy
3. Complete quick fixes (LICENSE, Cargo.toml)
4. Test publication locally
5. Publish to crates.io

