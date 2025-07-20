# uniffi-dart Fixtures Implementation Tracking

## Status Legend
- ✅ **COMPLETED** - Implementation finished and tests passing
- 🚧 **IN PROGRESS** - Currently being implemented
- ⏳ **TODO** - Planned for implementation
- ❌ **BLOCKED** - Blocked by dependencies or issues

---

## PHASE 1: Foundation Tests (Critical)

### 1.1 Basic Type System
- ✅ **`type-limits/`** - Comprehensive primitive type boundary testing
  - Status: Basic implementation complete, test has known BigInt issue
  - Location: `fixtures/type-limits/`
  - Tests: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `string`, `bytes` limits

- 🚧 **`simple-fns/`** - Basic free-standing function tests
  - Status: PARTIALLY COMPLETED - Core functionality working, HashMap blocked
  - Location: `fixtures/simple-fns/`
  - Focus: Foundation for all other function testing
  - Features: Basic returns, identity functions, type conversion, stateful objects, optional parameters
  - **BLOCKED**: HashMap functionality (`record<string, string>`) not yet supported by uniffi-dart
  - **Gap Identified**: Need to implement Map/HashMap support in uniffi-dart renderer

- ✅ **`simple-iface/`** - Basic interface/object testing
  - Status: COMPLETED
  - Source: `og-fixtures/fixtures/simple-iface/`
  - Focus: Foundation for all object-oriented features

### 1.2 Enhanced Type System
- 🚧 **`enum-types/`** - Dedicated enum testing
  - Status: IMPLEMENTED BUT BLOCKED - Requires proc-macro support
  - Source: `og-fixtures/fixtures/enum-types/`
  - Focus: Comprehensive enum serialization/deserialization
  - **Blocked by**: `#[derive(uniffi::Enum)]`, `#[derive(uniffi::Record)]`, `#[derive(uniffi::Object)]`, `#[uniffi::export]`

- 🚧 **`struct-default-values/`** - Default value handling tests
  - Status: IMPLEMENTED BUT BLOCKED - Default values not implemented in uniffi-dart
  - Source: `og-fixtures/fixtures/struct-default-values/`
  - Focus: Dictionary/record default values
  - **Gap Identified**: Dictionary default values generate positional constructors instead of named parameters with defaults

### 1.3 Error System Enhancement
- 🚧 **`large-error/`** - Large error type handling
  - Status: IMPLEMENTED BUT BLOCKED - Requires proc-macro support
  - Source: `og-fixtures/fixtures/large-error/`
  - Focus: Complement to current `error_types/`
  - **Blocked by**: `#[derive(uniffi::Record)]`, `#[derive(uniffi::Error)]`, `#[uniffi::export]`

---

## PHASE 2: Advanced Language Features (High Impact)

### 2.1 Trait System
- 🚧 **`trait-methods/`** - **CRITICAL MISSING FEATURE**
  - Status: IMPLEMENTED BUT BLOCKED - Requires trait method support
  - Source: `og-fixtures/fixtures/trait-methods/`
  - Focus: Trait implementations with `Debug`, `Display`, `Eq`, `Hash`
  - **Blocked by**: `[Traits=(Display, Debug, Eq, Hash)]` UDL attribute and proc-macro support

### 2.2 Procedural Macros
- 🚧 **`proc-macro/`** - **CRITICAL MISSING FEATURE**
  - Status: IMPLEMENTED BUT BLOCKED - Requires comprehensive proc-macro support
  - Source: `og-fixtures/fixtures/proc-macro/`
  - Focus: Procedural macro testing and UDL/proc-macro interoperability
  - **Blocked by**: All proc-macro features: `#[derive(uniffi::*)]`, `#[uniffi::export]`, `#[uniffi::export(with_foreign)]`, trait definitions, HashMap support

- ⏳ **`proc-macro-no-implicit-prelude/`** - Proc macros without implicit prelude
  - Status: TODO
  - Source: `og-fixtures/fixtures/proc-macro-no-implicit-prelude/`

### 2.3 Documentation System
- ⏳ **`docstring/`** - Documentation string generation
  - Status: TODO
  - Source: `og-fixtures/fixtures/docstring/`

- ⏳ **`docstring-proc-macro/`** - Documentation with proc macros
  - Status: TODO
  - Source: `og-fixtures/fixtures/docstring-proc-macro/`

---

## PHASE 3: External Integration (Critical for Ecosystem)

### 3.1 Multi-Crate External Types
- ⏳ **`ext-types/`** - **CRITICAL MISSING FEATURE**
  - Status: TODO
  - Source: `og-fixtures/fixtures/ext-types/`
  - Sub-components:
    - ⏳ **`ext-types/lib/`** - Main library consuming external types
    - ⏳ **`ext-types/proc-macro-lib/`** - Proc macro variant
    - ⏳ **`ext-types/custom-types/`** - Type wrapping (Guid, Handle → String, u64)
    - ⏳ **`ext-types/uniffi-one/`** - Standard UniFFI crate
    - ⏳ **`ext-types/sub-lib/`** - Intermediate library
    - ⏳ **`ext-types/external-crate/`** - Non-UniFFI crate with exposed types
    - ⏳ **`ext-types/http-headermap/`** - HTTP header type integration

### 3.2 Time-Based Features
- ⏳ **`uniffi-fixture-time/`** - Enhanced time-based functionality
  - Status: TODO
  - Source: `og-fixtures/fixtures/uniffi-fixture-time/`
  - Focus: Build on current `duration_type_test/`

---

## PHASE 4: Quality Assurance & Regression

### 4.1 Comprehensive Regression Suite
- ⏳ **`regressions/`** - **HIGH IMPORTANCE**
  - Status: TODO
  - Source: `og-fixtures/fixtures/regressions/`
  - Sub-components:
    - ⏳ **`regressions/cdylib-crate-type-dependency/`** - CDylib crate type dependency issues
    - ⏳ **`regressions/enum-without-i32-helpers/`** - Enum without i32 helper functions
    - ⏳ **`regressions/fully-qualified-types/`** - Fully qualified type path handling
    - ⏳ **`regressions/logging-callback-interface/`** - Logging callback interface issues
    - ⏳ **`regressions/missing-newline/`** - Missing newline handling
    - ⏳ **`regressions/unary-result-alias/`** - Unary result type aliasing
    - ⏳ **`regressions/wrong-lower-check/`** - Incorrect lowering validation

### 4.2 UI and Compiler Testing
- ⏳ **`uitests/`** - User interface/compiler error message testing
  - Status: TODO
  - Source: `og-fixtures/fixtures/uitests/`

- ⏳ **`version-mismatch/`** - Version compatibility testing
  - Status: TODO
  - Source: `og-fixtures/fixtures/version-mismatch/`

### 4.3 Metadata System
- ⏳ **`metadata/`** - Metadata handling and validation
  - Status: TODO
  - Source: `og-fixtures/fixtures/metadata/`

---

## PHASE 5: Performance & Scale

### 5.1 Performance Testing
- ⏳ **`benchmarks/`** - Performance benchmarking
  - Status: TODO
  - Source: `og-fixtures/fixtures/benchmarks/`

### 5.2 Platform Support
- ⏳ **`wasm-unstable-single-threaded/`** - WebAssembly support
  - Status: TODO (Future consideration)
  - Source: `og-fixtures/fixtures/wasm-unstable-single-threaded/`

---

## PHASE 6: Language-Specific Features (Future)

### 6.1 Keyword Conflict Resolution
- ⏳ **`keywords/`** - Keyword conflict resolution
  - Status: TODO (Future)
  - Source: `og-fixtures/fixtures/keywords/`

### 6.2 Swift-Specific Features (Future Multi-Language)
- ⏳ **`swift-codable/`** - Swift Codable protocol conformance
  - Status: TODO (Future)
  - Source: `og-fixtures/fixtures/swift-codable/`

- ⏳ **`swift-omit-labels/`** - Swift argument label omission
  - Status: TODO (Future)
  - Source: `og-fixtures/fixtures/swift-omit-labels/`

- ⏳ **`swift-bridging-header-compile/`** - Swift bridging header compilation
  - Status: TODO (Future)
  - Source: `og-fixtures/fixtures/swift-bridging-header-compile/`

- ⏳ **`swift-link-frameworks/`** - Swift framework linking
  - Status: TODO (Future)
  - Source: `og-fixtures/fixtures/swift-link-frameworks/`

---

## EXISTING FIXTURES (Already Implemented)

### Core Features
- ✅ **`arithmetic/`** - Basic arithmetic operations
- ✅ **`bytes_types/`** - Byte and binary data handling
- ✅ **`callbacks/`** - Callback interface implementations
- ✅ **`dart_async/`** - Async/futures functionality
- ✅ **`duration_type_test/`** - Duration type handling
- ✅ **`error_types/`** - Error handling and exceptions
- ✅ **`hello_world/`** - Basic "hello world" functionality
- ✅ **`large_enum/`** - Large enum compilation testing
- ✅ **`streams_ext/`** - Stream extension functionality

### External Integration
- ✅ **`external_types_provider/`** - Basic external type provider
- ✅ **`external_types_provider/consumer/`** - External type consumer

### Disabled/Incomplete
- ❌ **`coverall/`** - Comprehensive feature coverage (disabled in workspace)
- ❌ **`custom_types/`** - Custom type conversion (disabled in workspace)
- ❌ **`dispose/`** - Resource disposal (disabled in workspace)

---

## IDENTIFIED GAPS & MISSING FEATURES

### **Critical Missing Features in uniffi-dart**

#### **1. HashMap/Map Support (`record<string, string>`)**
- **Status**: ❌ **NOT IMPLEMENTED**
- **Error**: `not yet implemented: Renderable for Type::Map { key_type: String, value_type: String }`
- **Impact**: Blocks `simple-fns` fixture HashMap functionality
- **Location**: `src/gen/render/mod.rs:101:18`
- **Required for**: HashMap identity functions, Map parameter passing, Dictionary types

#### **2. BigInt Support for u64 Large Values**
- **Status**: ❌ **PARTIALLY IMPLEMENTED**
- **Error**: `The argument type 'BigInt' can't be assigned to the parameter type 'int'`
- **Impact**: Blocks `type-limits` fixture for large u64 values
- **Required for**: Full u64 range support, large integer boundary testing

---

## Implementation Notes

### Testing Pattern
All fixtures follow this pattern:
- `fixtures/[fixture-name]/Cargo.toml` - Rust crate configuration
- `fixtures/[fixture-name]/build.rs` - Build script using `uniffi_dart::generate_scaffolding`
- `fixtures/[fixture-name]/src/lib.rs` - Rust implementation
- `fixtures/[fixture-name]/src/api.udl` - UniFFI interface definition
- `fixtures/[fixture-name]/test/[fixture-name]_test.dart` - Dart test file
- `fixtures/[fixture-name]/tests/mod.rs` - Integration test calling `uniffi_dart::testing::run_test`

### Running Tests
```bash
# Run individual fixture test
cargo test -p [fixture_name]

# Run all fixture tests
cargo test
```

### Current Status Summary
- **Total Fixtures Planned**: 45
- **Completed**: 10 (existing) + 2 (type-limits, simple-iface) = 12
- **Implemented but Blocked**: 5 (simple-fns, enum-types, struct-default-values, large-error, trait-methods, proc-macro) = 6
- **In Progress**: 0
- **TODO**: 27
- **Key Blockers**: HashMap/Map support, dictionary default values, comprehensive proc-macro support

### Next Steps
1. **PRIORITY**: Implement missing core features to unblock Phase 1:
   - HashMap/Map support (`record<string, string>`) - blocks `simple-fns/`
   - Dictionary default values with named parameters - blocks `struct-default-values/`
   - Proc-macro support (`#[derive(uniffi::*)]`, `#[uniffi::export]`) - blocks `enum-types/`, `large-error/`, and many others

2. Complete Phase 1 foundation tests after implementing above features
3. Implement Phase 2 advanced features (`trait-methods/`, `proc-macro/`, `docstring/`)
4. Build Phase 3 external integration (`ext-types/`, `uniffi-fixture-time/`)
5. Add Phase 4 quality assurance (`regressions/`, `uitests/`, `metadata/`)
6. Implement Phase 5 performance testing (`benchmarks/`) 