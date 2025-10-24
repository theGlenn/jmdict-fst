# 🚀 jmdict-fast

> **Blazing-fast, Japanese dictionary engine**

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

> **Note:** This crate uses [bunpo](https://github.com/theGlenn/jmdict-fst/tree/main/bunpo) for Japanese conjugation handling. Both crates are part of the same monorepo but are published separately to crates.io.

---

## ✨ Features

- **💾 Compile-time indexed data** — FST + binary blob for maximum efficiency
- **⚡ Instant lookups** — O(log n) exact matching across all writing systems
- **🔎 Multimodal search** — Kanji, kana, and romaji support
- **📦 Ergonomic Rust API** — Usable as a library or binary
- **🪶 Tiny binary** — Zero runtime parsing, no allocations during lookup
- **🎯 Memory-mapped** — Zero-copy access to all dictionary data

---

## 🏎️ Performance at a Glance

| Metric           | Value                |
|------------------|---------------------|
| **Index Size**   | ~888KB (FSTs)       |
| **Data Size**    | 16MB binary blob    |
| **Entries**      | 22,569              |
| **Unique Keys**  | 24,342              |
| **Lookup Speed** | O(log n), instant   |
| **Memory Usage** | Memory-mapped, zero allocations |

---

## 🚀 Quick Start

### Building the Dictionary

```bash
cargo build
```

This creates:
- `OUT_DIR/kanji.fst` — Kanji lookup index
- `OUT_DIR/kana.fst` — Kana lookup index
- `OUT_DIR/romaji.fst` — Romaji lookup index
- `OUT_DIR/entries.bin` — Binary blob with all entries

### Using the Library

#### Search - Prefix
```rust
use jmdict_fast::Dict;

fn main() -> anyhow::Result<()> {
    let dict = Dict::load_default()?;
    let results = dict.lookup_partial("こんに");
    for entry in &results {
        println!("Found: {:?}", entry.kanji);
        println!("Reading: {:?}", entry.kana);
        println!("Meanings: {:?}", entry.sense[0].gloss);
    }
    Ok(())
}
```

#### Search Exact
```rust
use jmdict_fast::Dict;

fn main() -> anyhow::Result<()> {
    let dict = Dict::load_default()?;
    let results = dict.lookup_exact("こんにちは");
    for entry in &results {
        println!("Found: {:?}", entry.kanji);
        println!("Reading: {:?}", entry.kana);
        println!("Meanings: {:?}", entry.sense[0].gloss);
    }
    Ok(())
}
```
---

## 📊 Data Structure

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   kanji.fst     │    │   kana.fst      │    │  romaji.fst     │
│   (243KB)       │    │   (257KB)       │    │   (388KB)       │
│                 │    │                 │    │                 │
│ 漢字 → Entry ID  │    │ かな → Entry ID  │    │ romaji → Entry ID│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │  entries.bin    │
                    │    (16MB)       │
                    │                 │
                    │ Offset Table    │
                    │ + JSON Entries  │
                    └─────────────────┘
```

---

## 🔧 API Reference

- `Dict::load<P: AsRef<Path>>(base_dir: P) -> Result<Self>` — Loads the dictionary from the specified directory.
- `dict.lookup_exact(term: &str) -> Vec<Entry>` — Performs exact lookup across all writing systems.

**Entry Structure:**
```rust
pub struct Entry {
    pub id: String,                    // JMdict entry ID
    pub kanji: Vec<KanjiEntry>,        // Kanji forms
    pub kana: Vec<KanaEntry>,          // Kana readings
    pub sense: Vec<SenseEntry>,        // Meanings and metadata
}

pub struct KanjiEntry {
    pub common: bool,                  // Is this a common kanji?
    pub text: String,                  // The kanji text
    pub tags: Vec<String>,             // JMdict tags
}

pub struct KanaEntry {
    pub common: bool,                  // Is this a common reading?
    pub text: String,                  // The kana text
    pub tags: Vec<String>,             // JMdict tags
    pub applies_to_kanji: Vec<String>, // Which kanji this applies to
}

pub struct SenseEntry {
    pub part_of_speech: Vec<String>,   // Grammatical information
    pub applies_to_kanji: Vec<String>, // Which kanji this sense applies to
    pub applies_to_kana: Vec<String>,  // Which kana this sense applies to
    pub gloss: Vec<GlossEntry>,        // English translations
    // ... other JMdict fields
}
```

---

## 🛠️ Development

### Caching System

The build script implements a robust caching system to avoid re-downloading the large JMdict dataset. See [CACHING.md](./CACHING.md) and [CACHE_QUICK_REFERENCE.md](./CACHE_QUICK_REFERENCE.md) for details.

---

## 🔍 How It Works

1. **Build Phase:** The `build` tool processes the JMdict JSON and creates FST indexes and a binary blob for instant retrieval.
2. **Runtime Phase:** The library provides memory-mapped loading, FST-based lookups, and efficient entry retrieval.

---

## 📈 Real Benchmark Results

**Criterion (lookup_word.rs) — MacBook, Rust 1.70+**

```
lookup_exact 猫 (jmdict-fast)
    time:   [4.06 µs]
lookup_word 猫 (jmdict)
    time:   [511.96 µs]
```

- **jmdict-fast** is ~125x faster than a traditional filter-based approach for exact lookups.
- Both methods are stable, but jmdict-fast is highly optimized for speed and memory.

---

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- **JMdict** — The source dictionary data - see (EDRDG DICTIONARY LICENCE STATEMENT)[https://www.edrdg.org/edrdg/licence.html]
- **FST crate** — Fast finite state transducer implementation
- [10ten Japanese Reader](https://github.com/birchill/10ten-ja-reader) for their definflector implemtation
- **Rust ecosystem** — For making this possible

---

**Built with ❤️ and Rust** 🦀