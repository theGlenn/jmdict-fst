# AGENTS.md — jmdict-fast

Crate-level guidance for AI coding agents. See the [root AGENTS.md](../AGENTS.md) for workspace-wide rules.

## What this crate is

FST-backed Japanese dictionary engine. Three FSTs (kana, kanji, romaji) map keys → entry id; entry bodies live in `entries.bin` and are deserialized lazily on hit. There is also an `id.fst` for lookup by entry id.

Public surface is small and intentional — see `src/lib.rs`:

```rust
pub use dict::Dict;
pub use error::JmdictError;
pub use model::*;
pub use query::{BatchQueryBuilder, LookupResultIter, QueryBuilder};
```

`Dict` exposes both convenience methods (`lookup_exact`, `lookup_partial`, `lookup_exact_with_deinflection`) and a builder (`dict.lookup(term).mode(...).execute()`). The convenience methods are thin wrappers — keep them in sync with the builder if you change semantics.

## Module map

| Module | Role |
|---|---|
| `dict.rs` | `Dict` struct, file/embedded loading, FST candidate collection, entry materialization. |
| `model.rs` | `Entry`, `LookupResult`, `MatchType`, `MatchMode`, `DataVersion`, plus `MAGIC` / `FORMAT_VERSION` constants. |
| `query.rs` | `QueryBuilder` / `BatchQueryBuilder` — the public fluent API. |
| `error.rs` | `JmdictError`. |
| `tests.rs` | Integration-style tests that load real data via `Dict::load_default()`. |
| `build.rs` | No-op unless the `embedded` feature is on; then copies `dist/*` into `OUT_DIR`. |

## Commands

| Task | Command |
|---|---|
| Build (default features, runtime-loaded) | `cargo build -p jmdict-fast` |
| Build with embedded data | `cargo build -p jmdict-fast --features embedded` (needs `dist/`) |
| No-default-features check (CI runs this) | `cargo check -p jmdict-fast --no-default-features` |
| Tests | `cargo test -p jmdict-fast` (needs `dist/` — run `cargo xtask generate` first) |
| Benches | `cargo bench -p jmdict-fast` (`lookup_word`, `minimal`) |
| Examples | `cargo run -p jmdict-fast --example <name>` |

Tests call `Dict::load_default()`, which falls back to `../dist` via `CARGO_MANIFEST_DIR` in `#[cfg(test)]` builds. Don't break that fallback — workspace-relative `dist/` is how the local dev loop works.

## On-disk format invariants

`entries.bin` layout (little-endian):

```
[MAGIC: 4 bytes "JMDF"]
[FORMAT_VERSION: u32]
[jmdict_version: u16 len + utf8 bytes]
[generated_at:   u16 len + utf8 bytes]
[entry_count: u32]
[offset table: entry_count * (u32 offset, u32 len) = 8 bytes per entry]
[postcard-serialized entry bodies]
```

Rules:

- **`MAGIC` and `FORMAT_VERSION` are owned here** (`model.rs`). `xtask` re-uses them via `jmdict_fast::{MAGIC, FORMAT_VERSION}`. Do not duplicate the constants.
- **Bump `FORMAT_VERSION`** any time the header, offset table, or serialized `Entry` shape changes. `parse_entries_header` rejects mismatches with `JmdictError::DataVersionMismatch` — that's a feature, not something to relax.
- Entries are serialized with `postcard`; the `Entry` type's serde representation is part of the on-disk contract.
- The release tarball name (`jmdict-data-jmdict<X>-fmt<Y>.tar.gz`) encodes `FORMAT_VERSION` — consumers rely on this to pick the right asset.

## Loading semantics

`Dict::load_default()` tries, in order:

1. Embedded data (only when the `embedded` feature is on).
2. `$JMDICT_DATA` directory.
3. `dist/` relative to CWD.
4. *(tests only)* `../dist` relative to `CARGO_MANIFEST_DIR`.

Don't reorder these without thinking about downstream users — embedded must win when the feature is on, and `JMDICT_DATA` must beat the implicit `dist/` lookup.

## Scoring & ranking

Candidate scoring lives in `dict.rs`:

- `Exact` → `1.0`
- `Prefix` → `0.5`
- `Fuzzy` → length-aware, floor at `0.1` (`0.5 - (len_diff / (key_len + term_len)) * 0.2`)
- `Deinflected` → `0.75`

Prefix and fuzzy collection use `upsert_better` to keep the highest-scored hit per entry id across all three FSTs — a later exact hit in romaji must beat an earlier fuzzy hit in kana for the same id. Result ordering uses score desc, then `id` asc for determinism (`HashMap` iteration order is not stable). Preserve both properties.

## Dependencies — keep lean

This crate already pulls in `fst`, `memmap2`, `postcard`, `serde`, `simd-json`, `deunicode`, `unicode-normalization`, and `bunpo`. Don't add more without a strong reason — published binary size and compile time matter. In particular:

- No async runtime.
- No logging framework (the crate is silent by design).
- `bunpo` stays a regular dep — it's the deinflector behind `lookup_exact_with_deinflection`.

## Things to avoid

- Don't reintroduce unconditional `include_bytes!` in `build.rs` or `lib.rs`. Embedded mode is opt-in via the `embedded` feature; the v0.1.x migration documented in the root README is done.
- Don't hand-edit `CHANGELOG.md` or the crate `version` — `release-plz` handles both from conventional commits.
- Don't add `pub` to anything in `dict.rs` marked `pub(crate)` (e.g. `MatchCandidate`, the `*_candidates` helpers) without a deliberate API decision — they are internal scaffolding for `QueryBuilder`.
- Don't change `JmdictError` variants in a non-additive way without bumping the crate version appropriately (semver).
- Don't make `Dict` `Send`/`Sync` assumptions without checking — `Dict` holds `Cow<[u8]>` over either mmap-copied or borrowed bytes; the public type is `Dict<'a>` for a reason.

## When in doubt

Search the existing tests (`src/tests.rs`) for the behavior you're touching — the integration tests document the intended contract for ranking, deinflection, and the version-mismatch error path.
