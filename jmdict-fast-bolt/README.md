# jmdict-fast-bolt

> **[BoltFFI](https://boltffi.dev) bindings** for `jmdict-fast` — Swift, Kotlin, Java, C#, and WASM/TypeScript.

This crate is one of the per-generator consumers of [`jmdict-fast-ffi`](../jmdict-fast-ffi/) — the FFI-agnostic facade. It describes the facade externally for BoltFFI's proc-macro pipeline; the facade itself stays generator-neutral.

## What lives here

| Item | Why it's here, not in `jmdict-fast-ffi` |
|---|---|
| `#[data]` mirrors of `Entry`, `KanjiEntry`, `Xref`, `LookupResult`, `QueryOptions`, … | BoltFFI proc macros must sit on the type definition, not on a re-export. |
| `#[error] enum Error` | Same reason — the macro turns the variant into a typed exception in target languages. |
| `#[export] impl Dict` | Methods that BoltFFI exposes to Swift/Kotlin/Java/C#/WASM. The struct itself stays private; only the impl block is visible. |
| `From<facade::T> for T` conversions | Trivial mapping at the boundary; the bulk of the logic lives in the facade. |

Everything else (the actual dictionary engine, the FST loaders, the search algorithms) is in `jmdict-fast`. Touch this crate only when the FFI surface or BoltFFI-specific decoration needs to change.

## Generating bindings

Prereqs (per BoltFFI docs): Rust 1.70+, plus the per-target toolchain (Xcode 15+ for Apple, Android NDK for Android, JDK 8+ for Java, .NET SDK 10.0+ for C#, Node.js 18+ and `wasm-pack` for WASM).

```bash
# Install once.
cargo install boltffi_cli

# One-time per-workspace config.
boltffi init

# Generate everything.
boltffi pack all --release

# Or per-target.
boltffi pack apple    # → dist/apple/   (XCFramework + Swift bindings)
boltffi pack android  # → dist/android/ (JNI libraries + Kotlin bindings)
boltffi pack java     # → dist/java/    (JNI library + Java bindings)
boltffi pack csharp   # → dist/csharp/  (NuGet package + native assets)
boltffi pack wasm     # → dist/wasm/    (WASM module + TypeScript bindings)
```

The Rust crate is `crate-type = ["lib", "staticlib", "cdylib"]` — `lib` keeps it usable from regular Rust workflows (`cargo test` against `tests/smoke.rs` exercises the same Rust surface that BoltFFI exports), while `staticlib`/`cdylib` produce the C-ABI artifacts BoltFFI packs.

## Architecture context

```
jmdict-fast            ← engine: lifetimes, iterators, &str, generics
jmdict-fast-ffi        ← LCD facade: Arc handles, owned types, no lifetimes
jmdict-fast-bolt       ← THIS crate — BoltFFI annotations on mirrors of the facade
jmdict-fast-uniffi     ← (not yet built) — UDL describing the facade
jmdict-fast-frb        ← (not yet built) — flutter_rust_bridge scan
```

Each per-generator crate is expected to be ~300 lines of mirror types + a thin `#[export] impl` block. If a generator's needs push back on the facade, the change goes to `jmdict-fast-ffi` so every generator inherits it — never fork the engine per generator.
