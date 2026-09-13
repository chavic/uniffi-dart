# Local validation

Base: upstream main `e2dd2bb7180609184d0c70ddde0353f5e11d4c3b`.
Platform: Linux x64; Dart 3.13.0 stable; Rust 1.85.1; UniFFI 0.31.2.

The runner regenerated bindings from the compiled Rust fixture using this checkout's generator. The original generated component SHA-256 was `27a6e44f3a25000b05708fef815f4aff39a7d23870a4837e763f3e09708c6660`.

| Case | Observed result |
| --- | --- |
| Unmodified bindings, same-thread callback | Passed; original Dart state updated |
| Unmodified bindings, Rust worker/join callback | SIGABRT; `Cannot invoke native callback outside an isolate` |
| Relay, direct callback | Passed |
| Relay, actual Rust worker/join callback | Passed |
| Relay, eight concurrent Rust workers, ten calls each | All 80 callbacks accounted for; return sum and original Dart state match |
| Relay, foreign-handle clone on a worker | One clone and two frees; original Dart object invoked |
| Relay, declared error and success | Both roundtripped through generated conversions and status handling |
| Relay, nested Dart/Rust/Dart calls | Passed; both original Dart objects updated |
| Relay, 100 repeated worker calls | All passed; registries empty after every call |
| Mutated Dart callback returns the correct first value without changing state | Rejected with `owner state was not updated` |

All seven positive relay cases passed. Both negative controls failed in the required way. Every positive case verified an empty generated Dart callback registry, an empty native route registry and no remaining active relay. This is handle-lifecycle evidence, not a claim of general heap leak freedom.

Rust fixture Clippy passed with warnings denied. Rust formatting and whitespace checks passed. The runner checks subprocess exit codes and diagnostic text and enforces watchdogs; an unrelated failure or timeout cannot satisfy the crash control.

During harness development, a direct DynamicLibrary lookup opened a different library copy from the native-assets loader. This caused the thread/counter check to fail. Instrumentation now uses the same `@Native` asset ID as generated calls. That check is retained.

See README.md for reproduction and scope limits. The runner writes the complete results JSON and individual subprocess logs to `target/relay-results/` in this checkout. This remains a local fixture-specific prototype; production generator source is unchanged.

## Sharing the runner

The runner now selects desktop library names for Linux, macOS and Windows, permits fresh dependency downloads, and records host/toolchain details without requiring an `upstream` remote. Those portability changes are an invitation to test other 64-bit desktop hosts, not a claim of validated behavior there. Android/iOS need a separate harness.

A one-second watchdog control terminated the Dart child process and recorded `passed: false` and `timed_out: true` in the partial JSON report. It was not accepted as the expected VM-abort control. Runtime success also requires the expected `PASS` marker, not just exit code zero.

No downstream report of this crash has been identified in this investigation. The demonstrated failure remains our reproduction, rather than a confirmed BDK, Payjoin or BeyondTranslate incident.
