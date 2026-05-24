# Changelog

## 0.1.6

- **Fix consumer build (`failed to load manifest for dependency
  jmdict-fast-ffi`).** The published `rust/Cargo.toml` carried a
  `path = "../../../jmdict-fast-ffi"` dep that escapes the pub.dev
  tarball. Cargo doesn't fall back to crates.io when the path is
  unresolvable (the `version =` next to a path is only used when the
  crate itself is `cargo publish`-ed, which strips the path; pub.dev
  doesn't strip). Switched to a pure crates.io dep — now possible
  because `jmdict-fast-ffi 0.1.5` is published. Dev workflow loses the
  local-path shortcut: changes to `jmdict-fast-ffi` need a crates.io
  publish before the flutter crate sees them.

## 0.1.5

- **Fix broken pub.dev build.** The Rust binding crate now lives at
  `flutter_package/rust/` so it ships inside the published tarball.
  Previously the cargokit script reached for `../../` and landed
  outside the package (`PathNotFoundException: …/.pub-cache/…/Cargo.toml`),
  which made `flutter pub add jmdict_fast` fail at build time.
- **`JmdictFast.install()` one-call entry point.** Wraps
  `WidgetsFlutterBinding.ensureInitialized()`, `RustLib.init()`,
  cache-directory discovery via `path_provider`, and `Dict.installWith`
  into a single `await`. `cacheDir`, `source`, and `force` are still
  overridable for advanced users. Consumers no longer need to know that
  FRB exists to use the package.
- **`entryCountInt()` extension.** Returns a Dart `int` instead of
  the FRB-generated `BigInt` for the common case where the count fits
  in `2^53` (JMdict has ~200k entries).
- `path_provider` is now a direct dependency.

## 0.1.4

First public release. Version aligned with the underlying
`jmdict-fast` Rust crate so `Dict.install()` resolves to the
matching GitHub release tarball without consumers needing to track
two separate version axes.

- `Dict.install()` — one-call download + extract + load from the
  matching GitHub release tarball (JMdict 3.6.1, format v4).
- `Dict.installFromUrl(url)` and `Dict.installFromTarball(path)` for
  self-hosted mirrors and offline use.
- `initSdkCacheDir(path)` — process-global cache root, first-set-wins.
  Mandatory on iOS, Android, and WASM; supply a writable path from
  `path_provider.getApplicationSupportDirectory()` at startup.
- Lookup surface: `lookupExact`, `lookupPartial`,
  `lookupExactWithDeinflection`, `lookupGloss`, `lookupById`,
  `lookupWithOptions`, `lookupBatch`, `resolveXref`.
- Browsing: `get_(seqId)`, `iterEntries(start, count)`. (`get` is
  reserved in Dart, so FRB suffixes with `_`.)
- `QueryOptions` with `MatchMode`, `commonOnly`, POS / misc / field /
  dialect filters, `limit`, `maxDistance` for fuzzy.
- Typed `Error` enum (DataNotFound, DataVersionMismatch, DataCorrupted,
  InvalidQuery, Io, Deserialization, CacheDirRequired,
  CacheDirAlreadySet, Network).
