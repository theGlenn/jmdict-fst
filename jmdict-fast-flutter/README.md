# jmdict-fast-flutter

> **Flutter/Dart bindings** for `jmdict-fast` via [flutter_rust_bridge](https://cjycode.com/flutter_rust_bridge/) v2. The actual pub.dev package is at [`flutter_package/`](./flutter_package/) — published as `jmdict_fast` ([pub.dev/packages/jmdict_fast](https://pub.dev/packages/jmdict_fast)).

Two demo apps live in this directory:

- [`flutter_package/example/`](./flutter_package/example/) — a small validation app that consumes the published-style package layout via a path dep. The minimum proof that `flutter pub add jmdict_fast` works end-to-end.
- [`example/`](./example/) — a fuller search demo (exact / prefix / gloss modes) with its own Rust shim. Consumes the binding crate directly; useful for iterating on the FRB-generated surface.

Both call `JmdictFast.install()` on first launch — one `await` that handles `WidgetsFlutterBinding.ensureInitialized()`, `RustLib.init()`, cache-directory discovery via `path_provider`, and the ~21 MB download of the matching `jmdict-fast` release tarball. To launch either:

```sh
cd jmdict-fast-flutter/flutter_package/example   # or .../example/
flutter run -d macos      # or `-d chrome`, `-d ios`, `-d android`, etc.
```

This crate is the per-generator consumer of [`jmdict-fast-ffi`](../jmdict-fast-ffi/) (the FFI-agnostic facade) for Flutter and Dart. It exposes a flat, Dart-friendly surface that `flutter_rust_bridge_codegen` scans to emit:

- `flutter_package/rust/src/frb_generated.rs` — Rust glue (linked into the cdylib that ships with the host app).
- `flutter_package/lib/src/…` — Dart bindings (classes, enums, exception types).

## Layout

```
jmdict-fast-flutter/
├── example/                              ← FRB-codegen scaffold demo (separate Cargo workspace)
└── flutter_package/                      ← what gets published to pub.dev as `jmdict_fast`
    ├── pubspec.yaml                      ← Dart package metadata (`name: jmdict_fast`)
    ├── lib/
    │   ├── jmdict_fast.dart              ← Curated public Dart entry point
    │   └── src/                          ← (generated) — do not hand-edit
    ├── ios/, macos/, android/, linux/, windows/, cargokit/   ← platform build glue
    ├── example/                          ← validation app for publish smoke
    └── rust/                             ← vendored Rust binding crate (standalone Cargo workspace)
        ├── Cargo.toml                    ← cdylib + staticlib output
        ├── flutter_rust_bridge.yaml      ← codegen config
        ├── src/
        │   ├── lib.rs                    ← `pub mod api;` + `mod frb_generated;`
        │   └── api/
        │       ├── mod.rs                ← module entry point FRB scans
        │       ├── dictionary.rs         ← Dict opaque handle + methods
        │       ├── install.rs            ← InstallOptions + initSdkCacheDir
        │       ├── model.rs              ← records mirrored from the facade
        │       └── error.rs              ← `enum Error` → Dart exception
        └── tests/smoke.rs                ← Rust-side smoke against the exported surface
```

The Rust crate is vendored inside `flutter_package/` so the pub.dev
tarball is self-contained: cargokit's iOS/macOS/Android/Linux/Windows
build hooks reach the binding crate at `flutter_package/rust/` without
leaving the published `.pub-cache` extract. The crate is a **separate
Cargo workspace**, not a member of the outer `damascus-v1` workspace —
keep it self-contained (no `version.workspace = true`, no path deps that
escape the package) so consumers can build it in isolation.

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

# From the binding crate root:
cd jmdict-fast-flutter/flutter_package/rust
flutter_rust_bridge_codegen generate
# or, while developing:
flutter_rust_bridge_codegen generate --watch
```

The generated Dart code lands one directory up under
`flutter_package/lib/src/`. The hand-curated
`flutter_package/lib/jmdict_fast.dart` then `export`s the pieces of that
surface that consumers should see.

## Building the native library for the host app

You don't have to. [cargokit](https://github.com/irondash/cargokit) is vendored under `flutter_package/cargokit/` and wired through the platform-specific build files:

| Platform | Build hook | Triggers |
|---|---|---|
| iOS / macOS | `flutter_package/{ios,macos}/jmdict_fast.podspec` `script_phase` | `pod install` during `flutter build` |
| Android | `flutter_package/android/build.gradle` `apply from cargokit/gradle/plugin.gradle` | Gradle, during `flutter build apk` / `appbundle` |
| Linux / Windows | `flutter_package/{linux,windows}/CMakeLists.txt` `apply_cargokit(...)` | CMake, during the desktop build |

All five point at the binding crate at `flutter_package/rust/`. cargokit invokes the right `cargo build --target ...` per architecture, downloads any missing Rust toolchains via `rustup`, and produces the static/dynamic libs the plugin links. The first run on a clean machine takes a few minutes; subsequent runs are cached.

Web (WASM) isn't wired through cargokit yet — `flutter run -d chrome` works for the existing Dart-side scaffolding but the cdylib isn't compiled to wasm32 in the build matrix.

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
