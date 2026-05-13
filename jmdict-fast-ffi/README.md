# jmdict-fast-ffi

> **FFI-agnostic facade over [`jmdict-fast`](../jmdict-fast/).** One surface, many generators.

This crate exists so that bindings for `uniffi` (Swift/Kotlin), `flutter_rust_bridge` (Dart/Flutter), and `bolt-ffi` (Kotlin/Swift/C#/Python) share **one Rust API** instead of each rewriting its own. The facade reshapes `jmdict-fast` into types every generator can describe externally — no generator-specific macros live in this crate.

## Architecture

```
jmdict-fast            ← pure Rust library (lifetimes, iterators, &str, etc.)
jmdict-fast-ffi        ← this crate — FFI lowest common denominator
  ├─ jmdict-fast-uniffi ← .udl + scaffolding (not yet built)
  ├─ jmdict-fast-frb    ← FRB scan / wrapper (not yet built)
  └─ jmdict-fast-bolt   ← bolt IDL (not yet built)
```

Each per-generator crate **describes** the facade externally — it does not modify it. That keeps the surface honest: if a generator can't handle something here, the constraint is visible.

## What the facade enforces

These rules are the intersection of what every popular Rust FFI generator can describe:

- **No lifetimes** in public types
- **No generics** on public methods
- **Owned data** in/out: `String`, `Vec<T>`, no `&str`, no `&[T]`
- **`Arc<Self>`** for the `Dict` handle (uniffi requires it; FRB and bolt-ffi accept it)
- **Concrete enums** for errors — `io::Error` collapses into `Error::Io { message: String }`
- **`u32` for indices / limits**, never `usize` (varies by arch)
- **`Option<T>` instead of sentinels** (every modern generator handles nullable types)
- **No iterators** across the boundary — `iter_entries(start, count)` paginates instead

## Differences vs `jmdict-fast`

| `jmdict-fast` | `jmdict-fast-ffi` |
|---|---|
| `Dict::load(path) -> Result<Self, _>` | `Dict::load(String) -> Result<Arc<Self>, Error>` |
| Fluent `QueryBuilder` (`.mode().common_only().pos()…`) | Single `QueryOptions` record + `lookup_with_options` |
| `JmdictError::IoError(std::io::Error)` | `Error::Io { message: String }` |
| `iter_entries() -> EntryIter` | `iter_entries(start: u64, count: u64) -> Vec<Entry>` |
| `lookup_batch` returns `Vec<(String, Vec<LookupResult>)>` | `Vec<BatchResult { term, results }>` |
| `entry_count() -> usize` | `entry_count() -> u64` |

Data types (`Entry`, `KanjiEntry`, `Xref`, …) are re-exported unchanged — they're already plain POD with public fields and no lifetimes.

## Adding a new generator

Each per-generator crate is expected to:

1. Depend on `jmdict-fast-ffi` (this crate).
2. Describe the facade externally — UDL for uniffi, FRB scan for flutter_rust_bridge, bolt IDL for bolt-ffi.
3. Add its scaffolding (a `build.rs` hook, an FRB config file, etc.).
4. Stay thin. If a generator needs the facade to change, treat that as a constraint to negotiate across all generators, not a per-crate fork.

Expected size of each wrapper: ~100–300 lines.
