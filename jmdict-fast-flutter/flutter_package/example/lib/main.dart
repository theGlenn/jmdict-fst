import 'package:flutter/material.dart';
import 'package:jmdict_fast/jmdict_fast.dart';

Future<void> main() async {
  // `JmdictFast.install()` handles `WidgetsFlutterBinding.ensureInitialized()`,
  // `RustLib.init()`, cache-directory discovery, and the install in one call.
  runApp(const _App());
}

class _App extends StatelessWidget {
  const _App();

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'jmdict_fast validation',
      home: const _Home(),
    );
  }
}

class _Home extends StatefulWidget {
  const _Home();

  @override
  State<_Home> createState() => _HomeState();
}

class _HomeState extends State<_Home> {
  Dict? _dict;
  int? _entryCount;
  int? _hitsForCat;
  String? _error;
  bool _busy = false;

  Future<void> _install() async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final dict = await JmdictFast.install();
      // FRB lookups are all `Future`s — they run on a worker isolate so
      // the install + first query don't stall the UI.
      final count = await dict.entryCountInt();
      final hits = await dict.lookupExact(term: '猫');
      if (!mounted) return;
      setState(() {
        _dict = dict;
        _entryCount = count;
        _hitsForCat = hits.length;
        _busy = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = '$e';
        _busy = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final dict = _dict;
    return Scaffold(
      appBar: AppBar(title: const Text('jmdict_fast package validation')),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              if (dict == null) ...[
                const Text(
                  'Smoke test for the published `jmdict_fast` package.\n'
                  'First run downloads ~21 MB; cached afterwards.',
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 16),
                FilledButton.icon(
                  onPressed: _busy ? null : _install,
                  icon: _busy
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.download),
                  label: Text(_busy ? 'Installing…' : 'Install dictionary'),
                ),
                if (_error != null) ...[
                  const SizedBox(height: 16),
                  Text(_error!, style: TextStyle(color: Theme.of(context).colorScheme.error)),
                ],
              ] else ...[
                Text('Loaded $_entryCount entries.'),
                const SizedBox(height: 8),
                Text('$_hitsForCat hits for 猫.'),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
