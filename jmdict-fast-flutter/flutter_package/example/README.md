# jmdict_fast — pub.dev validation example

Smoke-test app for the published `jmdict_fast` package. Consumes the
parent package via a path dep, calls `JmdictFast.install()`, and
renders the entry count + a single `lookupExact('猫')` round-trip.

The point of this app is to prove the published-style layout works
end-to-end on first launch:

1. cargokit cross-compiles the Rust binding at `../rust/` for the host
   platform.
2. The Flutter plugin loads the resulting native lib.
3. `JmdictFast.install()` downloads the matching jmdict-fast tarball
   into the platform cache and mmaps it.
4. A real query returns hits.

If any link in that chain breaks, this app fails — that's the whole
purpose.

## Run

```sh
cd jmdict-fast-flutter/flutter_package/example
flutter pub get
flutter run -d <target>     # macos / chrome / your iPhone / android emulator
```

First run downloads ~21 MB; warm cache after that. See
[`../README.md`](../README.md) for the full package overview, or
[`../../README.md`](../../README.md) for the binding crate context.
