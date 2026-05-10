# jmdict-fst

A monorepo for **high-performance Japanese dictionary and grammar tools**.

[![jmdict-fast](https://img.shields.io/crates/v/jmdict-fast.svg)](https://crates.io/crates/jmdict-fast)
[![bunpo](https://img.shields.io/crates/v/bunpo.svg)](https://crates.io/crates/bunpo)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## Overview

This repository includes two Rust crates, published independently:

### [jmdict-fast](./jmdict-fast/)

A **blazing-fast Japanese dictionary engine** powered by FST (finite state transducer) indexing.

- Supports kanji, kana, and romaji lookups
- Achieves **O(log n)** search performance
- Uses memory-mapped data with zero allocations
- Built from the official **JMdict** dictionary dataset
- Two modes: **embedded** (data baked into binary) or **runtime-loaded** (from filesystem)

### [bunpo](./bunpo/)

A **lightweight deinflection engine** for Japanese verbs and adjectives.

- Rule-based conjugation reversal
- Zero external dependencies
- Integrated with `jmdict-fast` for conjugation-aware lookups

### xtask

Build tooling for generating dictionary data files from the JMdict source.

## Quick Start

### 1. Generate dictionary data

```bash
cargo xtask generate
```

This downloads JMdict and produces FST indexes and a binary blob in `dist/`.

Or download pre-built data from a [GitHub Release](https://github.com/theGlenn/jmdict-fst/releases):

```bash
# Download and extract pre-built data (replace versions with the latest release's asset)
mkdir -p dist
curl -L https://github.com/theGlenn/jmdict-fst/releases/latest/download/jmdict-data-jmdict3.6.1-fmt3.tar.gz | tar xz -C dist/
```

> The release asset is named `jmdict-data-jmdict<JMDICT_VERSION>-fmt<FORMAT_VERSION>.tar.gz`. Check the [Releases page](https://github.com/theGlenn/jmdict-fst/releases) for current values.

### 2. Use the library

Add to your `Cargo.toml`:

```toml
[dependencies]
jmdict-fast = "0.1.1"
bunpo = "0.1.1"  # Optional - only needed for conjugation handling
```

#### Runtime-loaded mode (default)

Point `JMDICT_DATA` to your data directory, or place data files in `dist/`:

```rust
use jmdict_fast::Dict;

fn main() -> anyhow::Result<()> {
    // Loads from JMDICT_DATA env var, or dist/ directory
    let dict = Dict::load_default()?;

    // Exact lookup
    let results = dict.lookup_exact("猫");
    for entry in &results {
        println!("{}: {}", entry.kanji[0].text, entry.sense[0].gloss[0].text);
    }

    // Prefix search
    let results = dict.lookup_partial("こんに");

    // With deinflection (finds 食べる from 食べます)
    let results = dict.lookup_exact_with_deinflection("食べます");

    Ok(())
}
```

#### Embedded mode (data baked into binary)

Enable the `embedded` feature to compile data directly into the binary:

```toml
[dependencies]
jmdict-fast = { version = "0.1.1", features = ["embedded"] }
```

```rust
use jmdict_fast::Dict;

fn main() -> anyhow::Result<()> {
    // Data is compiled in - no filesystem access needed
    let dict = Dict::load_embedded()?;
    let results = dict.lookup_exact("猫");
    Ok(())
}
```

> **Note:** The `embedded` feature requires data files in `dist/` at compile time. Run `cargo xtask generate` first.

## Migration from embedded-only API

Previous versions always embedded dictionary data via `include_bytes!` in the build script. The new architecture separates data generation from compilation:

1. **Data generation** is now handled by `cargo xtask generate` (not `build.rs`)
2. **Embedded mode** is opt-in via the `embedded` Cargo feature flag
3. **Runtime loading** is the new default - point `JMDICT_DATA` to your data directory or use `Dict::load("path/to/data")`
4. `Dict::load_default()` automatically tries: embedded (if feature enabled) → `JMDICT_DATA` env var → `dist/` directory

## Environment Variables

| Variable | Description |
|----------|-------------|
| `JMDICT_DATA` | Path to directory containing FST and entries.bin files |

## Feature Flags

| Feature | Description |
|---------|-------------|
| `embedded` | Bake dictionary data into the binary via `include_bytes!` |

## Releases

Releases are automated by [release-plz](https://release-plz.dev/). On every push to `main`, a "Release PR" is opened (or refreshed) that bumps versions in `Cargo.toml`, regenerates `CHANGELOG.md` from the conventional-commit history, and updates inter-crate dependencies. Merging that PR triggers the publish step, which:

1. Pushes Git tags for each released crate.
2. Runs `cargo publish` for each published crate in dependency order (`bunpo` → `jmdict-fast`).
3. Creates the matching GitHub Release, which in turn fires `release.yml` to build and attach the dictionary-data tarball (`jmdict-data-jmdict<X>-fmt<Y>.tar.gz`).

Required repository secrets:

| Secret | Purpose |
|---|---|
| `CARGO_REGISTRY_TOKEN` | Token from `cargo login` for publishing to crates.io. |
| `RELEASE_PLZ_TOKEN` | Optional GitHub PAT (or App token) used so Release PRs can trigger downstream workflows like `ci.yml`. Falls back to `GITHUB_TOKEN` when unset, but PRs opened with the default token will not trigger CI. |

`xtask` is marked `publish = false` and excluded from `release-plz.toml`, so it never gets bumped or published.

## License

MIT License - see [LICENSE](./LICENSE)

## Acknowledgments

- **JMdict** - The source dictionary data - see [EDRDG DICTIONARY LICENCE STATEMENT](https://www.edrdg.org/edrdg/licence.html)
- **FST crate** - Fast finite state transducer implementation
- [10ten Japanese Reader](https://github.com/birchill/10ten-ja-reader) for their deinflector implementation
