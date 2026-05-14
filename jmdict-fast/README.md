# 🚀 jmdict-fast

> **Blazing-fast Japanese dictionary engine, powered by FST indexing.**

[![jmdict-fast on crates.io](https://img.shields.io/crates/v/jmdict-fast.svg?label=jmdict-fast)](https://crates.io/crates/jmdict-fast)
[![docs.rs](https://docs.rs/jmdict-fast/badge.svg)](https://docs.rs/jmdict-fast)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A Rust library that turns the official **JMdict** dataset into memory-mapped FST indexes and serves lookups in **~4 µs**. Designed for Japanese readers, IMEs, language-learning tools, and anything that needs to look up words *fast*.

> **Note:** This crate uses [bunpo](https://github.com/theGlenn/jmdict-fst/tree/main/bunpo) for Japanese conjugation handling. Both crates live in the same monorepo but are published separately to crates.io.

---

## ✨ Features

- **⚡ Instant lookups** — O(log n) exact matching across kanji, kana, and romaji (~4 µs per lookup)
- **🔎 Multimodal search** — exact, prefix, fuzzy (edit-distance), and English-gloss reverse lookup
- **🪶 Memory-mapped** — zero-copy access on `load`; the kernel pages data in on demand, no upfront read into a `Vec`, no allocations during lookup
- **🧠 Deinflection-aware** — finds `食べる` from `食べます` via [bunpo](https://crates.io/crates/bunpo)
- **📦 Two loading modes** — embedded (compile-time) or runtime-loaded (filesystem)
- **🏷️ Full JMdict data** — antonyms, dialects, field tags, cross-references, JMdict IDs
- **🎯 Filterable queries** — by part-of-speech, misc tag, field, dialect, common-only, with limits
- **🆔 Stable lookup by JMdict ID** plus a sequential iterator over every entry

---

## 🏎️ Performance at a Glance

| Metric            | Value                          |
|-------------------|--------------------------------|
| **Index size**    | ~888 KB (FSTs)                 |
| **Data size**     | ~16 MB binary blob             |
| **Lookup speed**  | O(log n), ~4 µs                |
| **Memory usage**  | Memory-mapped, zero allocations |

### Side-by-side: `jmdict-fast` vs [`jmdict`](https://crates.io/crates/jmdict)

The bundled Criterion bench ([`benches/lookup_word.rs`](./benches/lookup_word.rs)) looks up `猫` against both crates on the same machine:

| Crate                     | Approach                                  | Time per lookup | Relative |
|---------------------------|-------------------------------------------|-----------------|----------|
| **`jmdict-fast` (this)**  | FST index + memory-mapped binary blob     | **~4.06 µs**    | **1×**   |
| [`jmdict`](https://crates.io/crates/jmdict) v2.x | Linear filter over `entries()` iterator | ~511.96 µs      | ~125× slower |

That's the gap between an O(log n) FST walk and an O(n) full-table scan — the bigger your dictionary, the wider the gap gets. Run `cargo bench -p jmdict-fast` to reproduce on your hardware.

> Comparisons are deliberately scoped to crates that solve the same problem (in-process JMdict lookup from Rust). If you're aware of another crate worth benching against, please open an issue or PR.

---

## 🚀 Getting Started

### 1. Generate or download dictionary data

Data files are **not** included in the crate. Generate them or download pre-built artifacts:

```bash
# Option A — generate from source (requires network access)
cargo xtask generate

# Option B — download pre-built data from GitHub Releases
# (asset name encodes JMdict + format versions; check Releases for current values)
mkdir -p dist
curl -L https://github.com/theGlenn/jmdict-fst/releases/latest/download/jmdict-data-jmdict3.6.1-fmt4.tar.gz \
  | tar xz -C dist/
```

This produces seven files in `dist/`: `kana.fst`, `kanji.fst`, `romaji.fst`, `id.fst`, `gloss.fst`, `entries.bin`, and `gloss_postings.bin`.

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
    for result in dict.lookup_exact("猫") {
        let entry = &result.entry;
        println!("{}: {}", entry.kanji[0].text, entry.sense[0].gloss[0].text);
    }

    // Prefix search
    let _ = dict.lookup_partial("こんに");

    // With deinflection (finds 食べる from 食べます)
    let _ = dict.lookup_exact_with_deinflection("食べます");

    // Reverse lookup by English gloss (multi-token = AND)
    let _ = dict.lookup_gloss("to eat");

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
let dict = jmdict_fast::Dict::load_embedded()?;
```

> Requires data files in `dist/` when building. Run `cargo xtask generate` first.

---

## 🔧 Loading Behavior

`Dict::load_default()` tries sources in order:

1. **Embedded data** (if `embedded` feature is enabled)
2. **`JMDICT_DATA`** env var — path to a directory with data files
3. **`dist/`** relative to the current directory
4. **`dist/`** relative to the workspace root

Or load from an explicit path:

```rust
let dict = jmdict_fast::Dict::load("/path/to/data")?;
```

| Variable | Description |
|---|---|
| `JMDICT_DATA` | Path to directory containing FST and `entries.bin` files |

| Feature | Description |
|---|---|
| `embedded` | Bake dictionary data into the binary via `include_bytes!` |

---

## 📊 Data Structure

```
kana.fst   kanji.fst   romaji.fst   id.fst        gloss.fst
   │           │            │          │              │
   └───────────┼────────────┘          │              ▼
               ▼                       │      gloss_postings.bin
         entries.bin ◄─────────────────┘     (per token: u32 count
      (postcard-serialized entries           followed by count × u64
       with version header)                   entry ids, little-endian)
```

- **FST maps** — sorted key→entry-id indexes for each writing system, plus a JMdict-ID index
- **entries.bin** — versioned binary blob (magic `JMDF` + format version + postcard-serialized entries)
- **gloss.fst + gloss_postings.bin** — English-gloss reverse-lookup index: tokens → byte offset into a postings file containing the matching entry-id sets

---

## 🔍 How It Works

1. **Build phase** — `cargo xtask generate` downloads JMdict, normalizes it, and emits the four FSTs + `entries.bin`.
2. **Runtime phase** — `Dict::load` memory-maps the FSTs, and lookups walk the FST to find an entry offset, then deserialize a single entry from `entries.bin`. No global parse, no allocations on the hot path.

---

## 📚 API Reference

### Loading

- `Dict::load(path)` — load from a specific directory
- `Dict::load_default()` — auto-detect data location
- `Dict::load_embedded()` — load compile-time embedded data (requires `embedded` feature)

### Lookups

- `dict.lookup_exact(term)` — exact match across kana, kanji, romaji
- `dict.lookup_partial(prefix)` — prefix search
- `dict.lookup_exact_with_deinflection(term)` — exact match with verb/adjective deinflection
- `dict.lookup_by_id(jmdict_id)` — fetch by stable JMdict ID (string)
- `dict.lookup_gloss("to eat")` — reverse lookup by English gloss (multi-token = AND)
- `dict.resolve_xref(&xref)` — walk `SenseEntry::related` / `antonym` to entries
- `dict.lookup(term)` — `QueryBuilder` with `mode`, `common_only`, `pos`, `misc`, `field`, `dialect`, `limit`, `max_distance`
- `dict.lookup_batch(terms)` — same builder, multiple terms at once

### Browsing

- `dict.get(seq_id)` — fetch by sequential (internal) index `0..entry_count()`
- `dict.iter_entries()` — lazy iterator over every entry
- `dict.entry_count()` / `dict.version()` — dictionary metadata

### Entry helpers

```rust
let entry = dict.lookup_exact("猫")[0].entry.clone();
entry.primary_kanji();   // Some("猫")
entry.primary_kana();    // Some("ねこ")
entry.headword();        // kanji if present, else kana
entry.is_common();
entry.glosses("eng");    // Iterator<Item = &str>
entry.parts_of_speech(); // Vec<&str>, distinct, first-seen order
```

### Entry structure

```rust
pub struct Entry {
    pub id: String,
    pub kanji: Vec<KanjiEntry>,
    pub kana: Vec<KanaEntry>,
    pub sense: Vec<SenseEntry>,
}
```

See [docs.rs](https://docs.rs/jmdict-fast) for the full API.

---

## 🤝 Contributing

Issues, PRs, and ideas welcome — especially around new lookup modes, query ergonomics, and data quality. Fork, branch, test, PR.

## 📄 License

MIT License — see [LICENSE](LICENSE).

## 🙏 Acknowledgments

- **JMdict** — the source dictionary data. See the [EDRDG dictionary licence statement](https://www.edrdg.org/edrdg/licence.html).
- **[fst](https://crates.io/crates/fst)** — the underlying finite-state-transducer crate.
- **[10ten Japanese Reader](https://github.com/birchill/10ten-ja-reader)** — for their deinflector implementation.

---

**Built with ❤️ and Rust** 🦀
