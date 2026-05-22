# jmdict_fast

Blazing-fast Japanese dictionary engine for Flutter, backed by Rust via
[flutter_rust_bridge](https://pub.dev/packages/flutter_rust_bridge).

## Quick start

```dart
import 'package:flutter/material.dart';
import 'package:jmdict_fast/jmdict_fast.dart';

Future<void> main() async {
  // One call. `JmdictFast.install()` handles:
  //   - WidgetsFlutterBinding.ensureInitialized()
  //   - RustLib.init() (flutter_rust_bridge)
  //   - cache-directory discovery via path_provider
  //   - download + extract + mmap of the dictionary tarball
  //
  // First run pulls ~21 MB from the matching jmdict-fast GitHub release.
  // Subsequent runs are warm-cache (mmap-only, milliseconds).
  final dict = await JmdictFast.install();

  print('Loaded ${await dict.entryCountInt()} entries');

  final hits = await dict.lookupExact(term: '猫');
  for (final r in hits) {
    print('${r.entry.kanji.first.text} → ${r.entry.sense.first.gloss.first.text}');
  }

  runApp(MyApp(dict: dict));
}
```

Override knobs for advanced consumers:

```dart
final dict = await JmdictFast.install(
  source: const InstallSource.url(url: 'https://my-mirror.example/jmdict.tar.gz'),
  cacheDir: '/var/jmdict-cache',
  force: true,
);
```

See `example/` for a full demo and `lib/jmdict_fast.dart` for the public
surface.

## Internals (not for pub.dev users)

This package is the published face of the `jmdict-fast-flutter` crate.
The native Rust library is cross-compiled per platform via
[cargokit](https://github.com/irondash/cargokit), which is vendored
under `cargokit/` and wired through the platform-specific build files:

| Platform | Build hook |
|----------|------------|
| iOS / macOS | `ios/jmdict_fast.podspec` / `macos/jmdict_fast.podspec` `script_phase` |
| Android | `android/build.gradle` `apply from cargokit/gradle/plugin.gradle` |
| Linux / Windows | `{linux,windows}/CMakeLists.txt` `apply_cargokit(...)` |

All five point at the Cargo crate at `rust/`. The Rust source lives
inside this package (rather than alongside `flutter_package/`) so the
published tarball is self-contained — cargokit reaches the binding
crate without leaving the `.pub-cache` extract.

The generated Dart bindings live under `lib/src/`; the public re-exports
live in `lib/jmdict_fast.dart`.
