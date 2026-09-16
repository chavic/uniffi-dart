# Owner-dispatch validation

This draft is based directly on main e2dd2bb7180609184d0c70ddde0353f5e11d4c3b. Production generator code is unchanged.

Environment: Linux x64, Dart 3.13.0, Rust 1.85.1, UniFFI 0.31.2. Both JIT and compiled AOT runs pass all 11 positive relay scenarios, the unmodified same-thread baseline, the expected wrong-thread abort, and the state-mutation rejection control: 14 runtime cases per execution mode. Build, generator and package setup checks are recorded separately in the reports.

| Added case | Original per-call queue | Persistent owner queue |
| --- | --- | --- |
| Nested native call reuses outer callback handle | Reaches test marker, then 12-second watchdog timeout | Returns 203 and updates the original object twice |
| Native callback after creating call returned | SendError panic in extern C; process abort (-6) | Returns 101 through the generated Dart callback; original state updated |
| 50 retained clone/free cycles | Not separately run | 50 clone callbacks, 100 free callbacks; no handles/routes left after each cycle |
| Close with pending request and listener notification | No close protocol | Close refuses; callback completes after yielding; final close succeeds |

The timeout was observed before changing the relay and was not accepted as a passing runtime case. The original retained-callback abort was checked for both the entry marker and SendError. The baseline source and logs are preserved in the root checkout under review/follow-ups/callback-owner-dispatch.

Every successful relay case checks an empty generated callback registry, zero native routes and no active relay. Final close additionally checks no queued jobs, no outstanding listener notification and closed native state before closing the Dart listener. This proves tested handle lifecycle behavior, not general heap leak freedom or forced-teardown safety.

All existing direct, worker/join, parallel, clone/free, declared-error, nested and repeated-call cases continue to pass in both modes. The new queue supports retained synchronous callbacks; it does not establish support for async Dart callback methods or cancellation.

Rust 1.85.1 Clippy for the fixture and all targets passes with warnings denied. Rust/Dart formatting and whitespace checks pass. AOT uses dart build cli so the native asset is bundled; the initial dart compile exe attempt omitted the native asset and failed the same-thread baseline before callback testing. That harness problem was corrected rather than accepted as an expected abort.

Recorded reports: [JIT](results/linux-x64-jit.json), [AOT](results/linux-x64-aot.json). Fresh runs on 2026-09-16 tested clean commit 816ea89 from the local experiment/callback-owner-dispatch branch. This draft carries the same fixture implementation. The subsequent changes update documentation, copy those reports and add the dedicated CI workflow.

New behavior is validated locally on Linux x64. The Intel macOS/iOS-simulator/Android-emulator results reported on #186 concern the earlier prototype. No new physical-device, ARM32, multi-isolate, async-cancellation or thread-affine-native validation is claimed.
