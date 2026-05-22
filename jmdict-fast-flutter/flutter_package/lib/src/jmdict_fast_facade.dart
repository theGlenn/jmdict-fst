import 'package:flutter/widgets.dart' show WidgetsFlutterBinding;
import 'package:path_provider/path_provider.dart' show getApplicationSupportDirectory;

import 'api/dictionary.dart';
import 'api/error.dart';
import 'api/install.dart';
import 'frb_generated.dart';

/// One-call entry point for the `jmdict_fast` Flutter package.
///
/// Wraps the bridge initialisation, cache-directory bootstrapping, and
/// download into a single `await`. The 90% case is:
///
/// ```dart
/// final dict = await JmdictFast.install();
/// ```
///
/// Advanced users can swap the source or pin the cache root:
///
/// ```dart
/// final dict = await JmdictFast.install(
///   source: const InstallSource.url(url: 'https://my-mirror.example/jmdict.tar.gz'),
///   cacheDir: '/var/jmdict-cache',
///   force: true,
/// );
/// ```
///
/// Manual control over the individual steps is still available via
/// [RustLib.init], [initSdkCacheDir], and the [Dict] static install
/// methods. Use [JmdictFast.install] unless you need that fine-grained
/// control.
/// Ergonomic wrappers around the FRB-generated [Dict] surface.
///
/// FRB lowers Rust `u64` to Dart `BigInt`, which is technically correct
/// but awkward for callers that just want a count. `entryCount` and
/// `entryCountInt` coexist: the former preserves the generated signature
/// for advanced users, the latter is the friendly default.
extension DictX on Dict {
  /// Convenience: return the total entry count as a Dart [int].
  ///
  /// JMdict has on the order of 200k entries today and will never realistically
  /// exceed `2^53`, so the BigInt round-trip is overkill for app code.
  Future<int> entryCountInt() async => (await entryCount()).toInt();
}

final class JmdictFast {
  const JmdictFast._();

  /// Initialises the Rust bridge, picks a writable cache directory, and
  /// downloads (or reuses) the dictionary tarball.
  ///
  /// - On a warm cache this is fast (mmap reload only).
  /// - On a cold cache this downloads ~21 MB; show a splash UI while it
  ///   resolves.
  ///
  /// Parameters:
  ///
  /// - [cacheDir]: override the cache root. Defaults to
  ///   `getApplicationSupportDirectory()` (Application Support on
  ///   iOS/macOS, internal files on Android, equivalent on Linux/Windows).
  /// - [source]: where to fetch the tarball from. Defaults to
  ///   [InstallSource.officialRelease], which resolves to the GitHub
  ///   release matching this package version.
  /// - [force]: re-extract even when the on-disk cache looks complete.
  static Future<Dict> install({
    String? cacheDir,
    InstallSource source = const InstallSource.officialRelease(),
    bool force = false,
  }) async {
    WidgetsFlutterBinding.ensureInitialized();
    if (!RustLib.instance.initialized) {
      await RustLib.init();
    }
    final resolvedCacheDir =
        cacheDir ?? (await getApplicationSupportDirectory()).path;
    try {
      await initSdkCacheDir(path: resolvedCacheDir);
    } on Error_CacheDirAlreadySet {
      // Idempotent: first-set-wins. A second `JmdictFast.install()` in
      // the same process — say in a test or after a hot restart — just
      // reuses whatever cache dir was registered the first time.
    }
    return Dict.installWith(
      options: InstallOptions(
        cacheDir: cacheDir,
        source: source,
        force: force,
      ),
    );
  }
}
