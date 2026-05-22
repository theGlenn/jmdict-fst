/// Public entry point for the `jmdict_fast` Flutter package.
///
/// Re-exports the FRB-generated surface so consumers only need
/// `import 'package:jmdict_fast/jmdict_fast.dart';` to reach `Dict`,
/// the install API, error types, and the data records.
///
/// Initialise the bridge once at app startup:
///
/// ```dart
/// await RustLib.init();
/// ```
///
/// Then register a writable cache directory before any install call:
///
/// ```dart
/// final dir = await getApplicationSupportDirectory();
/// initSdkCacheDir(path: dir.path);
/// final dict = await Dict.install();
/// ```
library;

export 'src/frb_generated.dart' show RustLib;

export 'src/api/dictionary.dart';
export 'src/api/error.dart';
export 'src/api/install.dart';
export 'src/api/model.dart';
