# jmdict-fast

> **Blazing-fast Japanese dictionary engine** powered by FST indexing

[![crates.io](https://img.shields.io/crates/v/jmdict-fast.svg)](https://crates.io/crates/jmdict-fast)
[![docs.rs](https://docs.rs/jmdict-fast/badge.svg)](https://docs.rs/jmdict-fast)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

> **Note:** This crate uses [bunpo](https://github.com/theGlenn/jmdict-fst/tree/main/bunpo) for Japanese conjugation handling. Both crates are part of the same monorepo but are published separately to crates.io.

---

## Features

- **O(log n) lookups** across kanji, kana, and romaji
- **Memory-mapped** FST indexes with zero-copy access
- **Deinflection** support via [bunpo](https://crates.io/crates/bunpo)
- **Two loading modes:** embedded (compile-time) or runtime (filesystem)
- **Full JMdict data** including antonyms, dialects, field tags, and cross-references

---

## Getting Started

### 1. Generate or download dictionary data

Data files are **not** included in the crate. Generate them or download pre-built artifacts:

```bash
# Option A: Generate from source (requires network access)
cargo xtask generate

# Option B: Download pre-built data from GitHub Releases
mkdir -p dist
curl -L https://github.com/theGlenn/jmdict-fst/releases/latest/download/jmdict-data.tar.gz | tar xz -C dist/
```

This produces five files in `dist/`: `kana.fst`, `kanji.fst`, `romaji.fst`, `id.fst`, `entries.bin`.

### 2. Add the dependency

```toml
[dependencies]
jmdict-fast = "0.1.1"
```

### 3. Use the library

#### Runtime-loaded mode (default)

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

#### Embedded mode

Bake data into your binary at compile time:

```toml
[dependencies]
jmdict-fast = { version = "0.1.1", features = ["embedded"] }
```

```rust
let dict = Dict::load_embedded()?;
```

> Requires data files in `dist/` when building. Run `cargo xtask generate` first.

---

## Loading behavior

`Dict::load_default()` tries sources in order:

1. **Embedded data** (if `embedded` feature is enabled)
2. **`JMDICT_DATA` env var** — path to a directory with data files
3. **`dist/`** relative to the current directory
4. **`dist/`** relative to the workspace root

You can also load from an explicit path:

```rust
let dict = Dict::load("/path/to/data")?;
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `JMDICT_DATA` | Path to directory containing FST and entries.bin files |

## Feature Flags

| Feature | Description |
|---------|-------------|
| `embedded` | Bake dictionary data into the binary via `include_bytes!` |

---

## Data Generation (`cargo xtask generate`)

The `xtask` crate handles downloading JMdict and building the FST indexes:

```bash
# Generate to default output (dist/)
cargo xtask generate

# Generate to a custom directory
cargo xtask generate --output /path/to/output
```

Pre-built data is also attached to [GitHub Releases](https://github.com/theGlenn/jmdict-fst/releases) as `jmdict-data.tar.gz`.

---

## Data Structure

```
kana.fst     kanji.fst     romaji.fst    id.fst
   │              │              │            │
   └──────────────┼──────────────┘            │
                  ▼                           │
            entries.bin ◄─────────────────────┘
         (postcard-serialized entries
          with version header)
```

- **FST maps** — sorted key→entry-id indexes for each writing system
- **entries.bin** — versioned binary blob (magic `JMDF` + format version + postcard-serialized entries)

---

## Migration from embedded-only API

Previous versions embedded dictionary data via `include_bytes!` in `build.rs`. The new architecture:

1. **Data generation** moved to `cargo xtask generate` (separate from compilation)
2. **Embedded mode** is now opt-in via the `embedded` feature flag
3. **Runtime loading** is the default — set `JMDICT_DATA` or place files in `dist/`
4. `Dict::load_default()` cascades: embedded → env var → filesystem

If you were using `Dict::load_default()` before, it continues to work — just generate data first with `cargo xtask generate`.

---

## API Reference

### Loading

- `Dict::load(path)` — Load from a specific directory
- `Dict::load_default()` — Auto-detect data location
- `Dict::load_embedded()` — Load compile-time embedded data (requires `embedded` feature)

### Lookups

- `dict.lookup_exact(term)` — Exact match across kana, kanji, romaji
- `dict.lookup_partial(prefix)` — Prefix search
- `dict.lookup_exact_with_deinflection(term)` — Exact match with verb/adjective deinflection

### Entry Structure

```rust
pub struct Entry {
    pub id: String,
    pub kanji: Vec<KanjiEntry>,
    pub kana: Vec<KanaEntry>,
    pub sense: Vec<SenseEntry>,
}
```

See [docs.rs](https://docs.rs/jmdict-fast) for full API documentation.

---

## Performance

| Metric           | Value                |
|------------------|---------------------|
| **Index Size**   | ~888KB (FSTs)       |
| **Data Size**    | ~16MB binary blob   |
| **Lookup Speed** | O(log n), ~4 us     |

---

## License

MIT License — see [LICENSE](LICENSE) for details.

## Acknowledgments

- **JMdict** — The source dictionary data - see [EDRDG DICTIONARY LICENCE STATEMENT](https://www.edrdg.org/edrdg/licence.html)
- **FST crate** — Fast finite state transducer implementation
- [10ten Japanese Reader](https://github.com/birchill/10ten-ja-reader) for their deinflector implementation
