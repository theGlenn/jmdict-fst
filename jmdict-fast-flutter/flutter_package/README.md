# jmdict_fast

Blazing-fast Japanese dictionary engine for Flutter, backed by Rust via
[flutter_rust_bridge](https://pub.dev/packages/flutter_rust_bridge).

## Quick start

```dart
import 'package:jmdict_fast/jmdict_fast.dart';
import 'package:path_provider/path_provider.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();

  // One-time: register a writable cache dir for the install pipeline.
  final dir = await getApplicationSupportDirectory();
  initSdkCacheDir(path: dir.path);

  // First run downloads ~21 MB from the matching jmdict-fast release
  // tarball into the app cache; subsequent runs are mmap-only.
  final dict = await Dict.install();
  print('Loaded ${dict.entryCount()} entries');

  final hits = dict.lookupExact(term: '猫');
  for (final r in hits) {
    print('${r.entry.kanji.first.text} → ${r.entry.sense.first.gloss.first.text}');
  }
}
```

See `example/` for a full demo and `lib/jmdict_fast.dart` for the public
surface.

## Internals (not for pub.dev users)

This package is the published face of the `jmdict-fast-flutter` crate at
the repo root. The native Rust library is cross-compiled per platform
via [cargokit](https://github.com/irondash/cargokit), which is vendored
under `cargokit/` and wired through the platform-specific build files:

| Platform | Build hook |
|----------|------------|
| iOS / macOS | `ios/jmdict_fast.podspec` / `macos/jmdict_fast.podspec` `script_phase` |
| Android | `android/build.gradle` `apply from cargokit/gradle/plugin.gradle` |
| Linux / Windows | `{linux,windows}/CMakeLists.txt` `apply_cargokit(...)` |

All five point at the Cargo crate at `../../`. The generated Dart
bindings live under `lib/src/`; the public re-exports live in
`lib/jmdict_fast.dart`.
