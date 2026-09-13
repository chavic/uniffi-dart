#!/usr/bin/env python3
"""Local-only generated-UniFFI relay experiment with crash and mutation controls."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import shutil
import signal
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--dart', default=os.environ.get('DART', 'dart'))
parser.add_argument('--toolchain', default='1.85.1')
args = parser.parse_args()
resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
fixture = Path(__file__).resolve().parents[1]
workspace = fixture.parents[1]
target = Path(os.environ.get('CARGO_TARGET_DIR', workspace / 'target' / 'relay-validation')).resolve()
output = workspace / 'target' / 'relay-results'
output.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, CARGO_TARGET_DIR=str(target), CARGO_NET_OFFLINE='true')
results = []

def command(label, argv, cwd=workspace, timeout=120, expected='success'):
    process = subprocess.Popen(argv, cwd=cwd, env=env, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, text=True, start_new_session=True)
    try:
        stdout, _ = process.communicate(timeout=timeout)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        stdout, _ = process.communicate()
        (output / f'{label}.log').write_text(stdout + '\nWATCHDOG TIMEOUT\n')
        raise RuntimeError(f'{label} timed out after {timeout}s')
    (output / f'{label}.log').write_text(stdout)
    code = process.returncode
    if expected == 'crash':
        ok = code != 0 and 'Cannot invoke native callback outside an isolate' in stdout
    elif expected == 'state-failure':
        ok = code != 0 and 'owner state was not updated' in stdout
    else:
        ok = code == 0
    results.append({'case': label, 'exit': code, 'expectation': expected, 'passed': ok})
    print(f'{label}: exit={code}, expected={expected}, passed={ok}', flush=True)
    if not ok:
        print(stdout[-6000:])
        raise RuntimeError(f'{label} failed its control expectation')
    return stdout

command('build', ['cargo', '+' + args.toolchain, 'build', '-p', 'uniffi-dart',
                 '-p', 'callback_thread_relay', '--features', 'uniffi-dart/binary'], timeout=900)
shared = target / 'debug' / 'libcallback_thread_relay.so'
config = output / 'uniffi.toml'
config.write_text('[bindings.dart]\npackage_name = "relay_probe"\ncdylib_name = "callback_thread_relay"\n')
generated = output / 'generated'
generated.mkdir(exist_ok=True)
command('generate', [str(target / 'debug' / 'uniffi_bindgen_dart'), '--library', str(shared),
                    '--out-dir', str(generated), '--config', str(config), '--no-format'])
component = generated / 'callback_thread_relay.dart'
original = component.read_text()

# Native symbol annotations alone enable the relay. The generated method bodies,
# argument/result converters, callback methods, clone/free code remain unchanged.
replacements = {
 'uniffi_callback_thread_relay_fn_init_callback_vtable_sink': 'relay_init_callback_vtable_sink',
 **{f'uniffi_callback_thread_relay_fn_func_{name}': f'relay_{name}' for name in [
     'call_bytes_direct', 'call_bytes_thread', 'call_bytes_parallel',
     'call_checked_thread', 'call_clone_thread']},
}
def relay_symbols(source):
    for name, symbol in replacements.items():
        matches = list(re.finditer(r'\bexternal\s+\S+\s+' + re.escape(name) + r'\s*\(', source))
        if len(matches) != 1:
            raise RuntimeError(f'Expected one native declaration for {name}, got {len(matches)}')
        start = source.rfind('@Native<', 0, matches[0].start())
        asset = source.index('assetId:', start, matches[0].start())
        source = source[:asset] + f"symbol: '{symbol}',\n  " + source[asset:]
    return source

for variant in ['baseline', 'relay', 'mutation']:
    package = output / variant
    if package.exists(): shutil.rmtree(package)
    shutil.copytree(generated, package)
    shutil.copy2(shared, package / shared.name)
    shutil.copy2(fixture / 'probe' / 'probe.dart', package / 'probe.dart')
    source = original if variant == 'baseline' else relay_symbols(original)
    # Read-only instrumentation to prove both native AND Dart registries drain.
    source += '\nint relayProbeHandleCount() => FfiConverterCallbackInterfaceSink._handleMap.relayProbeSize;\n'
    (package / component.name).write_text(source)
    runtime_matches = 0
    for path in package.rglob('*.dart'):
        content = path.read_text()
        marker = 'class UniffiHandleMap<T> {'
        if marker in content:
            runtime_matches += 1
            path.write_text(content.replace(marker, marker + '\nint get relayProbeSize => _map.length;', 1))
    if runtime_matches != 1: raise RuntimeError('Expected one generated handle map')
    if variant == 'mutation':
        # Same relay and result, but omit the Dart state update. If this passes,
        # the test would be accepting a stubbed result instead of actual state.
        path = package / 'probe.dart'
        content = path.read_text()
        assert content.count('return ++value;') == 1
        path.write_text(content.replace('return ++value;', 'return value + 1;'))
    (package / 'pubspec.yaml').write_text('''name: relay_probe
version: 0.0.0
environment:
  sdk: '>=3.10.0 <4.0.0'
dependencies:
  ffi: ^2.0.1
  code_assets: any
  hooks: any
''')
    (package / 'hook').mkdir(exist_ok=True)
    (package / 'hook' / 'build.dart').write_text('''import 'package:hooks/hooks.dart';
import 'package:code_assets/code_assets.dart';
void main(List<String> args) async {
  await build(args, (input, output) async {
    output.assets.code.add(CodeAsset(
      package: input.packageName,
      name: 'uniffi:callback_thread_relay',
      linkMode: DynamicLoadingBundled(),
      file: input.packageRoot.resolve('libcallback_thread_relay.so'),
    ));
  });
}
''')
    command(f'{variant}-pub', [args.dart, 'pub', 'get', '--offline'], cwd=package)
    if variant == 'baseline':
        command('baseline-direct', [args.dart, 'run', 'probe.dart', 'baseline-direct'], cwd=package)
        command('baseline-thread', [args.dart, 'run', 'probe.dart', 'baseline-thread'], cwd=package,
                timeout=20, expected='crash')
    elif variant == 'mutation':
        command('mutation-state', [args.dart, 'run', 'probe.dart', 'thread'], cwd=package,
                timeout=20, expected='state-failure')
    else:
        for case in ['direct', 'thread', 'parallel', 'clone', 'errors', 'nested', 'repeat']:
            command('relay-' + case, [args.dart, 'run', 'probe.dart', case], cwd=package, timeout=30)

report = {'base': subprocess.check_output(['git', 'merge-base', 'HEAD', 'upstream/main'], cwd=workspace, text=True).strip(),
          'generated_sha256': hashlib.sha256(original.encode()).hexdigest(),
          'symbol_overrides': replacements, 'cases': results}
(output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
print('All controls and relay cases passed. Logs:', output)
