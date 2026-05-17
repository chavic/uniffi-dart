import 'dart:typed_data';

import 'package:test/test.dart';
import '../custom_types.dart';

void main() {
  test('custom alias bytes and nested map helpers round trip', () {
    final response = getZenEngineResponse();

    expect(response.performance, equals('ready'));
    expect(response.result, orderedEquals([1, 2, 3]));
    expect(response.trace, isNotNull);
    expect(response.trace!['primary']!.id, equals('primary'));
    expect(response.trace!['primary']!.value, orderedEquals([4, 5, 6]));

    final manual = ZenEngineResponse(
      performance: 'manual',
      result: Uint8List.fromList([9, 8, 7]),
      trace: {
        'manual': ZenEngineTrace(
          id: 'manual',
          value: Uint8List.fromList([6, 5, 4]),
        ),
      },
    );
    final roundTrip = returnZenEngineResponse(response: manual);

    expect(roundTrip.performance, equals('manual'));
    expect(roundTrip.result, orderedEquals([9, 8, 7]));
    expect(roundTrip.trace!['manual']!.id, equals('manual'));
    expect(roundTrip.trace!['manual']!.value, orderedEquals([6, 5, 4]));
  });
}
