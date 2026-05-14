import 'package:flutter_test/flutter_test.dart';
import 'package:example/main.dart';
import 'package:example/src/rust/frb_generated.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());
  testWidgets('Demo app boots and renders the load panel', (
    WidgetTester tester,
  ) async {
    await tester.pumpWidget(const DemoApp());
    // App bar reflects the demo, not the FRB quickstart template.
    expect(find.text('jmdict-fast-flutter demo'), findsOneWidget);
    // Before a dictionary is loaded, the load panel's button is visible.
    expect(find.text('Load dictionary'), findsOneWidget);
  });
}
