# jmdict-fast-flutter

> **Flutter/Dart bindings** for `jmdict-fast` via [flutter_rust_bridge](https://cjycode.com/flutter_rust_bridge/) v2.

A working end-to-end demo lives in [`example/`](./example/) — a small Flutter app that calls `lookup_exact` / `lookup_partial` / `lookup_gloss` through Rust on macOS, iOS, Android, Linux, Windows, and the web. The example was scaffolded with `flutter_rust_bridge_codegen create example` and rewired to consume `jmdict-fast-flutter` via a path dep. To launch:
>
> ```sh
> cd jmdict-fast-flutter/example
> flutter run -d macos      # or `-d chrome`, `-d ios`, `-d android`, etc.
> ```
>
> The first screen asks for the path to the JMdict data directory (defaults to `../../dist` — run `cargo xtask generate` from the repo root if you haven't already).

This crate is the per-generator consumer of [`jmdict-fast-ffi`](../jmdict-fast-ffi/) (the FFI-agnostic facade) for Flutter and Dart. It exposes a flat, Dart-friendly surface that `flutter_rust_bridge_codegen` scans to emit:

- `src/frb_generated.rs` — Rust glue (linked into the cdylib that ships with the host app).
- `dart/lib/src/…` — Dart bindings (classes, enums, exception types).

## Layout

```
jmdict-fast-flutter/
├── Cargo.toml                  ← Rust crate with cdylib+staticlib output
├── flutter_rust_bridge.yaml    ← codegen config
├── src/
│   ├── lib.rs                  ← `pub mod api;` + (after first codegen) `mod frb_generated;`
│   └── api/
│       ├── mod.rs              ← module entry point FRB scans
│       ├── dictionary.rs       ← Dict opaque handle + methods
│       ├── model.rs            ← Records mirrored from the facade
│       └── error.rs            ← `enum Error` → Dart exception
├── tests/
│   └── smoke.rs                ← Rust-side smoke against the exported surface
└── dart/
    ├── pubspec.yaml            ← Dart package metadata (`name: jmdict_fast`)
    └── lib/
        ├── jmdict_fast.dart    ← Curated public Dart entry point
        └── src/                ← (generated) — do not hand-edit
```

The records in `src/api/model.rs` mirror the facade's types. The mirroring is unavoidable for the same reason as in `jmdict-fast-bolt`: FRB scans this crate's API module, and cross-crate scanning of re-exported types is fragile. Conversions are trivial `From` impls.

### sync vs async on `Dict`

The methods are split deliberately:

| async | Why |
|---|---|
| `load`, `load_default` | I/O — mmap several files. |
| `lookup_gloss` | Posting-list intersection can hit thousands of entries for common tokens. |
| `lookup_with_options`, `lookup_batch` | Fuzzy / large filters / many terms can blow the 16 ms frame budget. |
| `resolve_xref` | Routes through `lookup_exact` + filtering; bounded but unbounded-feeling for callers. |
| `iter_entries` | Caller can ask for arbitrary `count`. |

| sync | Why |
|---|---|
| `lookup_exact`, `lookup_partial`, `lookup_exact_with_deinflection`, `lookup_by_id`, `get` | Microsecond-cost FST hits — forcing every Dart consumer to `await` would just add noise. |
| `entry_count`, `version` | One field read. |

FRB v2 runs `async fn` on a worker thread pool by default, so the async methods surface as `Future<T>` in Dart and never block the UI isolate.

## Generating bindings

```bash
# Install the codegen once.
cargo install flutter_rust_bridge_codegen

# From this crate's root:
flutter_rust_bridge_codegen generate
# or, while developing:
flutter_rust_bridge_codegen generate --watch
```

After the first run, add `mod frb_generated;` below `pub mod api;` in `src/lib.rs` so the generated glue is part of the crate's compilation unit.

The generated Dart code lands under `dart/lib/src/`. The hand-curated `dart/lib/jmdict_fast.dart` then `export`s the pieces of that surface that consumers should see.

## Building the native library for the host app

Each Flutter target wants its own platform-specific binary:

- **iOS / macOS**: `cargo build --release --target aarch64-apple-ios` (etc.), then assemble an XCFramework that the Flutter plugin embeds.
- **Android**: `cargo build --release --target aarch64-linux-android` (etc.), copy `.so` into `android/src/main/jniLibs/<abi>/`.
- **Desktop**: `cargo build --release` produces a `.so`/`.dylib`/`.dll` for the host platform.
- **Web**: `wasm-pack build --target web` (requires the `wasm-start` FRB feature).

Once the FRB Flutter plugin scaffolding is in place (one of the things `flutter_rust_bridge_codegen create` produces in fresh projects), the host Flutter app picks the right binary automatically.

## Why the codegen step is not in CI yet

`flutter_rust_bridge_codegen generate` mutates files (creates `src/frb_generated.rs` and the Dart `src/` directory). Running it in CI before any change is reviewed conflicts with the committed scaffold. Plan: once the Dart consumer is in active use, add a check job that runs `generate --no-write` (or compares the generated output against a golden) and fails if drift exists.

## Architecture context

```
jmdict-fast              ← engine: lifetimes, iterators, &str, generics
jmdict-fast-ffi          ← LCD facade: owned types, Arc handles, no lifetimes
jmdict-fast-flutter      ← THIS crate — FRB-scanned mirror + Dict class
jmdict-fast-bolt         ← Swift/Kotlin/Java/C#/WASM via BoltFFI
jmdict-fast-uniffi       ← (not yet built) — UDL describing the facade
```
