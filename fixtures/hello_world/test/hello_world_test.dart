import 'package:test/test.dart';
import '../hello_world.dart';

void main() {
  test('hello world', () {
    expect(helloWorld(), "hello world");
  });

  test('hello mikka', () {
    expect(hello("mikka"), "hello mikka");
  });

  test("object test", () {
    final world = newWorld();
    expect(world.isThere(), true);
  });

  test("stringed world test", () {
    var world = newWorldWithName("sari");
    expect(world.name(), "sari");
    expect(world.prefixedName("mister"), "mister sari");
    expect(world.prefixedName(null), null);
    world = world.setName("new name");
    expect(world.name(), "new name");
    world = world.setName(null);
    expect(world.name(), null);
  });
}
