# jmdict-fst

> **Blazing-fast Japanese dictionary lookups, powered by FST indexing.**

[![jmdict-fast](https://img.shields.io/crates/v/jmdict-fast.svg)](https://crates.io/crates/jmdict-fast)
[![docs.rs](https://docs.rs/jmdict-fast/badge.svg)](https://docs.rs/jmdict-fast)
[![bunpo](https://img.shields.io/crates/v/bunpo.svg)](https://crates.io/crates/bunpo)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

`jmdict-fst` is a monorepo built around **[jmdict-fast](./jmdict-fast/)** — a Rust dictionary engine that turns the official **JMdict** dataset into memory-mapped FST indexes and serves lookups in **~4 µs**.

If you're building a Japanese reader, an IME, a language-learning app, or anything that needs to look up words *fast* — this is for you.

---

## ✨ Features

- **⚡ Instant lookups** — O(log n) exact matching across kanji, kana, and romaji (~4 µs per lookup)
- **🔎 Multimodal search** — exact, prefix, fuzzy, and English-gloss reverse lookup
- **🪶 Memory-mapped** — zero-copy access, no upfront read into a `Vec`, no allocations during lookup
- **🧠 Deinflection-aware** — finds `食べる` from `食べます` via the bundled [bunpo](./bunpo/) deinflector
- **📦 Two loading modes** — embedded (data baked into the binary) or runtime-loaded (from filesystem)
- **🏷️ Full JMdict data** — antonyms, dialects, field tags, cross-references, JMdict IDs
- **🎯 Filterable queries** — by part-of-speech, misc tag, field, dialect, common-only, with limits and edit distance

---

## 🏎️ Performance at a Glance

| Metric            | Value                          |
|-------------------|--------------------------------|
| **Index size**    | ~888 KB (FSTs)                 |
| **Data size**     | ~16 MB binary blob             |
| **Lookup speed**  | O(log n), ~4 µs                |
| **Memory usage**  | Memory-mapped, zero allocations |

### Side-by-side vs [`jmdict`](https://crates.io/crates/jmdict)

The bundled Criterion bench ([`jmdict-fast/benches/lookup_word.rs`](./jmdict-fast/benches/lookup_word.rs)) looks up `猫` against both crates on the same machine:

| Crate                                              | Approach                                | Time per lookup | Relative      |
|----------------------------------------------------|-----------------------------------------|-----------------|---------------|
| **`jmdict-fast` (this)**                           | FST index + memory-mapped binary blob   | **~4.06 µs**    | **1×**        |
| [`jmdict`](https://crates.io/crates/jmdict) v2.x   | Linear filter over `entries()` iterator | ~511.96 µs      | ~125× slower  |

That's the gap between an O(log n) FST walk and an O(n) full-table scan. Run `cargo bench -p jmdict-fast` to reproduce.

---

## 🚀 Quick Start

### 1. Get the dictionary data

Data files are not bundled with the crate. Generate them locally or grab a pre-built tarball:

```bash
# Option A — generate from source (requires network access)
cargo xtask generate

# Option B — download pre-built data from GitHub Releases
mkdir -p dist
curl -L https://github.com/theGlenn/jmdict-fst/releases/latest/download/jmdict-data-jmdict3.6.1-fmt3.tar.gz \
  | tar xz -C dist/
```

> Release assets are named `jmdict-data-jmdict<JMDICT_VERSION>-fmt<FORMAT_VERSION>.tar.gz`. The current format version is **4** (adds the English-gloss reverse-lookup index). Check the [Releases page](https://github.com/theGlenn/jmdict-fst/releases) for current values.

### 2. Add the dependency

```toml
[dependencies]
jmdict-fast = "0.1.1"
```

### 3. Look things up

```rust
use jmdict_fast::Dict;

fn main() -> anyhow::Result<()> {
    // Loads from JMDICT_DATA env var, or dist/ directory
    let dict = Dict::load_default()?;

    // Exact lookup
    for entry in dict.lookup_exact("猫") {
        println!("{}: {}", entry.kanji[0].text, entry.sense[0].gloss[0].text);
    }

    // Prefix search
    let _ = dict.lookup_partial("こんに");

    // Deinflection-aware (finds 食べる from 食べます)
    let _ = dict.lookup_exact_with_deinflection("食べます");

    // Reverse lookup by English gloss
    let _ = dict.lookup_gloss("to eat");

    Ok(())
}
```

### Embedded mode (data baked into the binary)

```toml
[dependencies]
jmdict-fast = { version = "0.1.1", features = ["embedded"] }
```

```rust
let dict = jmdict_fast::Dict::load_embedded()?;
```

> Requires data files in `dist/` at compile time. Run `cargo xtask generate` first.

See the **[jmdict-fast crate README](./jmdict-fast/)** for the full API reference.

---

## 📦 Repository Layout

The core of the project is **[`jmdict-fast`](./jmdict-fast/)**. Everything else is either a supporting library or a higher-level wrapper.

| Crate | Role |
|---|---|
| **[`jmdict-fast`](./jmdict-fast/)** | The dictionary engine. The thing you probably want. |
| [`bunpo`](./bunpo/) | Lightweight, zero-dependency deinflection engine. Used by `jmdict-fast` for conjugation handling, but also publishable on its own. |
| [`jmdict-fast-ffi`](./jmdict-fast-ffi/) | FFI-agnostic facade crate — the foundation for non-Rust bindings. |
| [`jmdict-fast-bolt`](./jmdict-fast-bolt/) | BoltFFI bindings for Swift, Kotlin, Java, C#, and WASM. |
| [`jmdict-fast-flutter`](./jmdict-fast-flutter/) | Flutter bindings via `flutter_rust_bridge`, with an end-to-end example app. |
| `xtask` | Build tooling: downloads JMdict and produces the FST indexes + binary blob. |

---

## 🔧 Loading Behavior

`Dict::load_default()` tries sources in order:

1. **Embedded data** (if the `embedded` feature is enabled)
2. **`JMDICT_DATA`** env var — path to a directory with data files
3. **`dist/`** relative to the current directory
4. **`dist/`** relative to the workspace root

You can also load from an explicit path: `Dict::load("/path/to/data")?`.

| Variable | Description |
|---|---|
| `JMDICT_DATA` | Path to directory containing FST and `entries.bin` files |

| Feature | Description |
|---|---|
| `embedded` | Bake dictionary data into the binary via `include_bytes!` |

---

## 📦 Releases

Releases are automated by [release-plz](https://release-plz.dev/). On every push to `main`, a "Release PR" is opened (or refreshed) that bumps versions in `Cargo.toml`, regenerates `CHANGELOG.md` from the conventional-commit history, and updates inter-crate dependencies. Merging that PR triggers the publish step, which:

1. Pushes Git tags for each released crate.
2. Runs `cargo publish` for each published crate in dependency order (`bunpo` → `jmdict-fast`).
3. Creates the matching GitHub Release, which in turn fires `release.yml` to build and attach the dictionary-data tarball (`jmdict-data-jmdict<X>-fmt<Y>.tar.gz`).

Required repository secrets:

| Secret | Purpose |
|---|---|
| `CARGO_REGISTRY_TOKEN` | Token from `cargo login` for publishing to crates.io. |
| `RELEASE_PLZ_TOKEN` | GitHub PAT (or App token) with `pull-requests: write` and `contents: write`. Effectively required for the end-to-end flow above: GitHub does not fire downstream workflows from events created by the default `GITHUB_TOKEN`, so without this secret the Release PR will not run `ci.yml` and the published GitHub Release will not run `release.yml` (the dictionary-data tarball will not be attached automatically). The workflow falls back to `GITHUB_TOKEN` if unset, which is fine if you intend to attach the tarball by hand. |

`xtask` is marked `publish = false` and excluded from `release-plz.toml`, so it never gets bumped or published.

---

## 🤝 Contributing

Issues, PRs, and ideas welcome — especially around new lookup modes, FFI ergonomics, or data quality. Fork, branch, test, PR.

## 📄 License

MIT License — see [LICENSE](./LICENSE).

## 🙏 Acknowledgments

- **JMdict** — the source dictionary data. See the [EDRDG dictionary licence statement](https://www.edrdg.org/edrdg/licence.html).
- **[fst](https://crates.io/crates/fst)** — the underlying finite-state-transducer crate.
- **[10ten Japanese Reader](https://github.com/birchill/10ten-ja-reader)** — for their deinflector implementation, which inspired `bunpo`.

---

**Built with ❤️ and Rust** 🦀
