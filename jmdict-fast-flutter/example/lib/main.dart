import 'dart:async';

import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';

import 'src/rust/api/simple.dart';
import 'src/rust/frb_generated.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  // Register a writable cache dir for `Dict::install*`. The native side
  // can't compute this on iOS/Android (sandboxed), so the host supplies
  // it once at startup. On desktop this still works — supplying it is
  // strictly safer than letting the Rust default resolver pick.
  final dir = await getApplicationSupportDirectory();
  initCacheDir(path: dir.path);
  runApp(const DemoApp());
}

class DemoApp extends StatelessWidget {
  const DemoApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'jmdict-fast-flutter demo',
      theme: ThemeData(useMaterial3: true, colorSchemeSeed: Colors.indigo),
      home: const HomePage(),
    );
  }
}

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

enum _Mode { exact, partial, gloss }

class _HomePageState extends State<HomePage> {
  final _queryCtl = TextEditingController(text: '猫');

  BigInt? _entryCount;
  String? _loadError;
  bool _loading = false;
  List<DemoHit> _results = const [];
  Duration? _lastLookup;
  _Mode _mode = _Mode.exact;

  Future<void> _install() async {
    setState(() {
      _loading = true;
      _loadError = null;
    });
    try {
      // First run downloads ~21 MB from the GitHub release matching this
      // build's crate / JMdict / format version. Cached after that, so the
      // second launch is mmap-only.
      final count = await installDictionary();
      if (!mounted) return;
      setState(() {
        _entryCount = count;
        _loading = false;
      });
      _runQuery();
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _loadError = '$e';
        _loading = false;
      });
    }
  }

  Future<void> _runQuery() async {
    if (!isDictionaryReady()) return;
    // Snapshot the query parameters at the moment we kick off the lookup.
    // `lookupGloss` is async — if the user types again (or flips the mode)
    // while it's in flight, we don't want a stale result to overwrite the
    // newer one when the older Future finally resolves.
    final term = _queryCtl.text;
    final mode = _mode;
    final stopwatch = Stopwatch()..start();
    final hits = switch (mode) {
      _Mode.exact => lookupExact(term: term),
      _Mode.partial => lookupPartial(prefix: term),
      _Mode.gloss => await lookupGloss(query: term),
    };
    stopwatch.stop();
    if (!mounted || term != _queryCtl.text || mode != _mode) return;
    setState(() {
      _results = hits;
      _lastLookup = stopwatch.elapsed;
    });
  }

  @override
  Widget build(BuildContext context) {
    final loaded = _entryCount != null;
    return Scaffold(
      appBar: AppBar(
        title: const Text('jmdict-fast-flutter demo'),
        bottom: loaded
            ? PreferredSize(
                preferredSize: const Size.fromHeight(28),
                child: Container(
                  width: double.infinity,
                  padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
                  color: Theme.of(context).colorScheme.surfaceContainerHighest,
                  child: Text(
                    '${_entryCount} entries loaded · lookup '
                    '${_lastLookup != null ? '${_lastLookup!.inMicroseconds} µs' : '—'}',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ),
              )
            : null,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: loaded ? _buildSearch() : _buildLoadPanel(),
      ),
    );
  }

  Widget _buildLoadPanel() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        const Text(
          'First run downloads ~21 MB of JMdict data into the app cache.\n'
          'Cached afterwards — subsequent launches are mmap-only.',
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 24),
        FilledButton.icon(
          onPressed: _loading ? null : _install,
          icon: _loading
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Icon(Icons.download),
          label: Text(_loading ? 'Installing…' : 'Install dictionary'),
        ),
        if (_loadError != null) ...[
          const SizedBox(height: 16),
          Text(_loadError!, style: TextStyle(color: Theme.of(context).colorScheme.error)),
        ],
      ],
    );
  }

  Widget _buildSearch() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            Expanded(
              child: TextField(
                controller: _queryCtl,
                decoration: const InputDecoration(
                  border: OutlineInputBorder(),
                  hintText: 'Try 猫 / たべ / cat …',
                ),
                onSubmitted: (_) => _runQuery(),
              ),
            ),
            const SizedBox(width: 8),
            FilledButton(onPressed: _runQuery, child: const Text('Search')),
          ],
        ),
        const SizedBox(height: 8),
        SegmentedButton<_Mode>(
          segments: const [
            ButtonSegment(value: _Mode.exact, label: Text('Exact')),
            ButtonSegment(value: _Mode.partial, label: Text('Prefix')),
            ButtonSegment(value: _Mode.gloss, label: Text('Gloss')),
          ],
          selected: {_mode},
          onSelectionChanged: (s) {
            setState(() => _mode = s.first);
            _runQuery();
          },
        ),
        const SizedBox(height: 16),
        Expanded(
          child: _results.isEmpty
              ? const Center(child: Text('No results.'))
              : ListView.separated(
                  itemCount: _results.length,
                  separatorBuilder: (_, _) => const Divider(height: 1),
                  itemBuilder: (_, i) {
                    final hit = _results[i];
                    return ListTile(
                      title: Text(hit.kanji ?? hit.kana ?? '(no headword)'),
                      subtitle: Text(
                        [
                          if (hit.kanji != null && hit.kana != null) hit.kana,
                          if (hit.gloss != null) hit.gloss,
                        ].whereType<String>().join(' · '),
                      ),
                    );
                  },
                ),
        ),
      ],
    );
  }
}
