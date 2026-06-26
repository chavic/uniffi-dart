# Changelog

All notable changes to uniffi-dart will be documented in this file.

## v0.2.1+v0.31.2

### Changes

- Upgrade uniffi-rs from v0.31.1 to v0.31.2 (#146)

### Fixes

- Fix name mangling for all-caps acronym identifiers (#145)
- Register recursive type helpers so recursive types generate correctly (#138)
- Generate Rust-backed proxies for callback trait interfaces and handle
  tagged callback interface pointers (#135)
- Make the ordering of generated Dart type helpers deterministic (#134)

## v0.2.0+v0.31.1

### Breaking changes

- **BREAKING**: Upgrade uniffi-rs from v0.30.0 to v0.31.1 (#125)

### Fixes

- Generate FfiConverter for Sequence/Optional of records in callback interfaces (#117)
- Fix Dart error object renaming (#120)
- Use Optional FfiConverter's `lower()` for inner types to avoid double-wrapping nullable records in callback return values (#121)
- Make nullable record fields optional in Dart constructors (#122)

## v0.1.0+v0.30.0

Initial release of uniffi-dart targeting uniffi-rs v0.30.0.

### Dart binding generation

- All primitive types with bounds checking
- Strings, bytes, optionals, sequences, and maps
- Records with default values and named constructors
- Enums (flat and complex) with variant support
- Objects with constructors, methods, and disposable pattern
- Error types and exception handling
- Custom types
- Durations and timestamps

### Async support

- Async/Future support for functions, methods, and constructors
- Callback interfaces (UDL and proc-macro)
- Trait interfaces
- Stream support via extension macros

### Code generation

- Named parameters for generated functions and objects
- Multiple namespace support
- Dart Native Assets with `@Native` annotations
- Configurable library loading strategy
- Formatted generated Dart code

### Testing

- Comprehensive test suite
- CI with downstream testing (rust-payjoin and bdk-dart)
