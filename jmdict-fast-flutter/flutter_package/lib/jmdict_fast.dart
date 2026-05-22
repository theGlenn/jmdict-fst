/// Public entry point for the `jmdict_fast` Flutter package.
///
/// The one-call install path:
///
/// ```dart
/// import 'package:jmdict_fast/jmdict_fast.dart';
///
/// void main() async {
///   final dict = await JmdictFast.install();
///   runApp(MyApp(dict: dict));
/// }
/// ```
///
/// [JmdictFast.install] handles `WidgetsFlutterBinding.ensureInitialized()`,
/// `flutter_rust_bridge` initialisation, cache-directory discovery via
/// `path_provider`, and the download in one call. On a warm cache it
/// resolves in milliseconds; on a cold cache it pulls ~21 MB.
///
/// Advanced users that need fine-grained control can still call the
/// individual primitives — [RustLib.init], [initSdkCacheDir], and the
/// [Dict] static `install*` methods — directly.
library;

export 'src/jmdict_fast_facade.dart' show JmdictFast, DictX;
export 'src/frb_generated.dart' show RustLib;

export 'src/api/dictionary.dart';
export 'src/api/error.dart';
export 'src/api/install.dart';
export 'src/api/model.dart';
