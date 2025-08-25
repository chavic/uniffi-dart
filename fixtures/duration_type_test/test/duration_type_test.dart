import 'package:test/test.dart';
import '../duration_type_test.dart';

void main() {
  test('rust return value seconds check', () {
    final duration = makeDuration(BigInt.from(5), 0);

    expect(duration.inSeconds, 5);
    expect(getSeconds(duration), BigInt.from(5));
    expect(getNanos(duration), 0);
  });

  test('seconds data check from dart', () {
    final duration = Duration(seconds: 10);
    expect(getSeconds(duration), BigInt.from(10));
    expect(getNanos(duration), 0);
  });

  test('check nanos/micros', () {
    final duration = makeDuration(BigInt.zero, 3000);
    expect(duration.inSeconds, 0);
    expect(duration.inMicroseconds, 3);
    expect(getSeconds(duration), BigInt.zero);
    expect(getNanos(duration), 3000);
  });

  test('check large values', () {
    final duration = makeDuration(BigInt.from(123456789), 3000000);
    expect(duration.inSeconds, 123456789);
    expect(duration.inMicroseconds, 123456789003000);
    expect(getSeconds(duration), BigInt.from(123456789));
    expect(getNanos(duration), 3000000);
  });
}
