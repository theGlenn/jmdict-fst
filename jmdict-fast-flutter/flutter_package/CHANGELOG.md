# Changelog

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
- Browsing: `get(seqId)`, `iterEntries(start, count)`.
- `QueryOptions` with `MatchMode`, `commonOnly`, POS / misc / field /
  dialect filters, `limit`, `maxDistance` for fuzzy.
- Typed `Error` enum (DataNotFound, DataVersionMismatch, DataCorrupted,
  InvalidQuery, Io, Deserialization, CacheDirRequired,
  CacheDirAlreadySet, Network).
