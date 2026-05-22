# jmdict-fast-flutter — search demo

Fuller demo app for the FRB binding crate. Has its own Rust shim
(`rust/src/api/simple.rs`) so it can experiment with the surface
freely without touching the published `jmdict_fast` package.

Shows off the core lookup modes the binding exposes:

- **Exact** — `lookupExact` across kanji / kana / romaji.
- **Prefix** — `lookupPartial` (starts-with).
- **Gloss** — `lookupGloss` (English reverse lookup; multi-token = AND).

A toggle in the UI flips between them; the header shows the entry
count + last lookup time in microseconds.

## Run

```sh
cd jmdict-fast-flutter/example
flutter pub get
flutter run -d macos        # or -d chrome / your iPhone / android
```

First launch calls `installDictionary()` which downloads the matching
jmdict-fast release tarball into the app's cache; subsequent launches
are mmap-only.

## Two demo apps?

Yes. [`../flutter_package/example/`](../flutter_package/example/) is
the minimal validation app that proves `flutter pub add jmdict_fast`
works — it consumes the published-style package via path dep. This
app, by contrast, consumes the binding crate directly and is useful
for iterating on the FRB-generated surface without going through pub.
