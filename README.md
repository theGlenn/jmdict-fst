# jmdict-fst

> **Blazing-fast Japanese dictionary lookups, powered by FST indexing.**

[![jmdict-fast on crates.io](https://img.shields.io/crates/v/jmdict-fast.svg?label=jmdict-fast)](https://crates.io/crates/jmdict-fast)
[![docs.rs](https://docs.rs/jmdict-fast/badge.svg)](https://docs.rs/jmdict-fast)
[![bunpo on crates.io](https://img.shields.io/crates/v/bunpo.svg?label=bunpo)](https://crates.io/crates/bunpo)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**SDK bindings:**
[![Swift / SPM](https://img.shields.io/badge/SPM-coming%20soon-lightgrey?logo=swift&logoColor=white)](#-repository-layout)
[![Kotlin / Maven](https://img.shields.io/badge/Kotlin-coming%20soon-lightgrey?logo=kotlin&logoColor=white)](#-repository-layout)
[![Flutter / pub.dev](https://img.shields.io/pub/v/jmdict_fast.svg?label=jmdict_fast&logo=flutter&logoColor=white)](https://pub.dev/packages/jmdict_fast)
[![Python / PyPI](https://img.shields.io/badge/PyPI-coming%20soon-lightgrey?logo=python&logoColor=white)](#-repository-layout)
[![JavaScript / npm](https://img.shields.io/badge/npm-coming%20soon-lightgrey?logo=npm&logoColor=white)](#-repository-layout)

`jmdict-fst` is a monorepo built around **[jmdict-fast](./jmdict-fast/)**, a Rust dictionary engine that turns the official **JMdict** dataset into memory-mapped FST indexes and serves lookups in **~4 µs**.

If you're building a Japanese reader, an IME, a language-learning app, or anything else that needs fast word lookup, this is for you.

---

## ✨ Features

- **⚡ Instant lookups**: O(log n) exact matching across kanji, kana, and romaji (~4 µs per lookup)
- **🔎 Multimodal search**: exact, prefix, fuzzy, and English-gloss reverse lookup
- **🪶 Memory-mapped**: zero-copy access, no upfront read into a `Vec`, no allocations during lookup
- **🧠 Deinflection-aware**: finds `食べる` from `食べます` via the bundled [bunpo](./bunpo/) deinflector
- **📦 Two loading modes**: embedded (data baked into the binary) or runtime-loaded (from filesystem)
- **🏷️ Full JMdict data**: antonyms, dialects, field tags, cross-references, JMdict IDs
- **🎯 Filterable queries**: by part-of-speech, misc tag, field, dialect, common-only, with limits and edit distance

---

## 🏎️ Performance at a Glance

| Metric            | Value                          |
|-------------------|--------------------------------|
| **Index size**    | ~888 KB (FSTs)                 |
| **Data size**     | ~16 MB binary blob             |
| **Lookup speed**  | O(log n), ~4 µs                |
| **Memory usage**  | Memory-mapped, zero allocations |

<details>
<summary><strong>Side-by-side vs other Rust JMdict crates</strong></summary>

Benched on the same machine looking up `猫`. Criterion bench is in [`jmdict-fast/benches/lookup_word.rs`](./jmdict-fast/benches/lookup_word.rs); the startup / RSS numbers come from the four standalone binaries in [`jmdict-fast/examples/startup_*.rs`](./jmdict-fast/examples/), each run as a fresh process.

#### Steady-state lookup (warm, after load)

| Crate                                            | Approach                                | Per-lookup    |
|--------------------------------------------------|-----------------------------------------|---------------|
| [`jisho`](https://crates.io/crates/jisho)        | Eager-loaded `HashMap`s (in-memory)     | **~48 ns**    |
| **`jmdict-fast` (this)**                         | FST index + mmap                        | ~3.1 µs       |
| [`jmdict`](https://crates.io/crates/jmdict)      | Linear filter over `entries()`          | ~549 µs       |

#### Time to first result (cold process start to first lookup returned)

| Crate                                | Cold first lookup | Warm first lookup |
|--------------------------------------|-------------------|-------------------|
| **`jmdict-fast`** (embedded)         | **~80 µs**        | **~13 µs**        |
| **`jmdict-fast`** (mmap)             | ~3 ms             | ~13 µs            |
| `jmdict`                             | ~5 ms             | ~880 µs           |
| `jisho`                              | ~82 ms            | ~54 ms            |

- `jisho`'s `lazy_static` decompresses and decodes the full dictionary on the first lookup. Every process invocation pays this cost.
- For CLI tools, mobile apps, serverless, and anything short-lived, `jmdict-fast` returns its first result roughly **1000 to 6000 times faster**.

#### Resident memory and binary size

| Crate                       | RSS after first lookup | Binary size       |
|-----------------------------|------------------------|-------------------|
| **`jmdict-fast`** (mmap)    | **~1.9 MB**            | **0.5 MB** + ~17 MB external data |
| **`jmdict-fast`** (embedded)| **~1.8 MB**            | ~48 MB (data baked in)            |
| `jmdict`                    | ~6.9 MB                | ~5.9 MB (English-only subset)     |
| `jisho`                     | ~100 MB                | ~85 MB                            |

`jmdict-fast` uses roughly **50 times less memory** than `jisho`, because the FST and entry blob are memory-mapped. The OS pages in only what you touch.

#### Feature matrix

| Capability              | `jmdict-fast` | `jisho` | `jmdict` |
|-------------------------|---------------|---------|----------|
| Exact lookup            | ✅            | ✅      | ✅       |
| Prefix search           | ✅            | wildcards only | ❌ |
| Fuzzy / Levenshtein     | ✅            | ❌      | ❌       |
| English-gloss reverse   | ✅            | ✅      | (via filter) |
| Deinflection            | ✅            | ❌      | ❌       |
| Cross-platform bindings | Swift / Kotlin / Flutter (and more in progress) | ❌ | ❌ |

#### TL;DR

- Pick **`jisho`** for long-lived servers doing millions of exact lookups where RAM is free.
- Pick **`jmdict-fast`** for mobile, CLI tools, serverless, and IMEs. Anything short-lived, memory-constrained, or that needs more than exact lookup.

To reproduce: run `cargo bench -p jmdict-fast --features embedded` for the lookup benchmark, and `cargo run --release -p jmdict-fast --example startup_<engine>` for the startup and RSS numbers.

</details>

---

## 🚀 Quick Start

### 1. Get the dictionary data

Three ways, from least to most setup:

```rust
// Option A: let the library do it — downloads the matching tarball
// into the platform cache and loads it. Needs the `install` feature.
let dict = jmdict_fast::Dict::install()?;
```

```bash
# Option B: generate from source (one-time, requires network)
cargo xtask generate
```

```bash
# Option C: download the prebuilt tarball yourself
mkdir -p dist
curl -L https://github.com/theGlenn/jmdict-fst/releases/latest/download/jmdict-data-jmdict3.6.1-fmt4.tar.gz \
  | tar xz -C dist/
```

> Release assets are named `jmdict-data-jmdict<JMDICT_VERSION>-fmt<FORMAT_VERSION>.tar.gz`. The current format version is **4** (adds the English-gloss reverse-lookup index). Check the [Releases page](https://github.com/theGlenn/jmdict-fst/releases) for current values.

### 2. Add the dependency

```toml
[dependencies]
# Add the `install` feature if you want Option A (the auto-download
# path) from above. Option B/C work without it.
jmdict-fast = { version = "0.1.4", features = ["install"] }
```

### 3. Look things up

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
jmdict-fast = { version = "0.1.4", features = ["embedded"] }
```

```rust
let dict = jmdict_fast::Dict::load_embedded()?;
```

> Requires data files in `dist/` at compile time. Run `cargo xtask generate` first.

See the **[jmdict-fast crate README](./jmdict-fast/)** for the full API reference.

---

## 📱 Flutter

`jmdict_fast` is on pub.dev: [pub.dev/packages/jmdict_fast](https://pub.dev/packages/jmdict_fast).

```sh
flutter pub add jmdict_fast
```

```dart
import 'package:jmdict_fast/jmdict_fast.dart';

Future<void> main() async {
  // One call. JmdictFast.install handles WidgetsFlutterBinding,
  // flutter_rust_bridge init, cache-directory discovery (via
  // path_provider, bundled with the package), and the download.
  // First run pulls ~21 MB; subsequent runs are mmap-only.
  final dict = await JmdictFast.install();
  final hits = await dict.lookupExact(term: '猫');
  print('${hits.length} hits for 猫 across ${await dict.entryCountInt()} entries');
}
```

The package is built directly on `jmdict-fast` via [flutter_rust_bridge](https://pub.dev/packages/flutter_rust_bridge) — no on-device database, no first-launch import step. See [`jmdict-fast-flutter/flutter_package/example/`](./jmdict-fast-flutter/flutter_package/example/) for a runnable validation app and [`jmdict-fast-flutter/example/`](./jmdict-fast-flutter/example/) for a fuller search demo with prefix / gloss modes.

---

## 📦 Repository Layout

The core of the project is **[`jmdict-fast`](./jmdict-fast/)**. Everything else is either a supporting library or a higher-level wrapper.

| Crate | Role |
|---|---|
| **[`jmdict-fast`](./jmdict-fast/)** | The dictionary engine. The thing you probably want. |
| [`bunpo`](./bunpo/) | Lightweight, zero-dependency deinflection engine. Used by `jmdict-fast` for conjugation handling, but also publishable on its own. |
| [`jmdict-fast-ffi`](./jmdict-fast-ffi/) | FFI-agnostic facade crate. The foundation for non-Rust bindings. |
| [`jmdict-fast-bolt`](./jmdict-fast-bolt/) | BoltFFI bindings for Swift, Kotlin, Java, C#, and WASM. |
| [`jmdict-fast-flutter`](./jmdict-fast-flutter/) | Flutter bindings via `flutter_rust_bridge`. The published Dart package lives at [`flutter_package/`](./jmdict-fast-flutter/flutter_package/) with the Rust binding crate vendored under [`flutter_package/rust/`](./jmdict-fast-flutter/flutter_package/rust/) so the pub.dev tarball is self-contained — [pub.dev/packages/jmdict_fast](https://pub.dev/packages/jmdict_fast). |
| `xtask` | Build tooling: downloads JMdict and produces the FST indexes + binary blob. |

---

## 🔧 Loading Behavior

`Dict::load_default()` tries sources in order:

1. **Embedded data** (if the `embedded` feature is enabled)
2. **`JMDICT_DATA`** env var: path to a directory with data files
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

**Pub.dev (Flutter)** is a manual step. After a crate release ships, run `cd jmdict-fast-flutter/flutter_package && flutter pub publish` from a machine with pub credentials. The Dart package version generally tracks the Rust crate version, but they can drift independently — e.g. 0.1.5 was a Dart-only hotfix for a pub.dev build path.

---

## 🤝 Contributing

Issues, PRs, and ideas welcome, especially around new lookup modes, FFI ergonomics, or data quality. Fork, branch, test, PR.

## 📄 License

MIT License. See [LICENSE](./LICENSE).

## 🙏 Acknowledgments

- **JMdict**: the source dictionary data. See the [EDRDG dictionary licence statement](https://www.edrdg.org/edrdg/licence.html).
- **[fst](https://crates.io/crates/fst)**: the underlying finite-state-transducer crate.
- **[10ten Japanese Reader](https://github.com/birchill/10ten-ja-reader)**: for their deinflector implementation, which inspired `bunpo`.

---

**Built with ❤️ and Rust** 🦀
