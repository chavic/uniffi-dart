use uniffi;
use uniffi::{Enum, Record};

#[uniffi::export]
pub fn new_flat_one() -> FlatEnum {
    FlatEnum::One
}

#[uniffi::export]
pub fn new_flat_two() -> FlatEnum {
    FlatEnum::Two
}

#[uniffi::export]
pub fn new_flat_three() -> FlatEnum {
    FlatEnum::Three
}

#[uniffi::export]
pub fn new_flat_four() -> FlatEnum {
    FlatEnum::Four
}

#[uniffi::export]
pub fn take_flat_enum(flat: FlatEnum) -> String {
    match flat {
        FlatEnum::One => "One".to_string(),
        FlatEnum::Two => "Two".to_string(),
        FlatEnum::Three => "Three".to_string(),
        FlatEnum::Four => "Four".to_string(),
    }
}

#[uniffi::export]
pub fn new_u8_value(value: u8) -> Value {
    Value::U8 { value }
}

#[uniffi::export]
pub fn new_i8_value(value: i8) -> Value {
    Value::I8 { value }
}

#[uniffi::export]
pub fn new_u16_value(value: u16) -> Value {
    Value::U16 { value }
}

#[uniffi::export]
pub fn new_i16_value(value: i16) -> Value {
    Value::I16 { value }
}

#[uniffi::export]
pub fn new_u64_value(value: u64) -> Value {
    Value::U64 { value }
}

#[uniffi::export]
pub fn new_i64_value(value: i64) -> Value {
    Value::I64 { value }
}

#[uniffi::export]
pub fn new_u32_value(value: u32) -> Value {
    Value::U32 { value }
}

#[uniffi::export]
pub fn new_i32_value(value: i32) -> Value {
    Value::I32 { value }
}

#[uniffi::export]
pub fn new_f32_value(value: f32) -> Value {
    Value::F32 { value }
}

#[uniffi::export]
pub fn new_f64_value(value: f64) -> Value {
    Value::F64 { value }
}

#[uniffi::export]
pub fn new_string_value(value: String) -> Value {
    Value::String { value }
}

#[uniffi::export]
pub fn new_bool_value(value: bool) -> Value {
    Value::Bool { value }
}
// Holding off till refactor
#[uniffi::export]
pub fn new_public_key_value_without_argument() -> Value {
    Value::PublicKey {
        value: vec![3, 4, 4, 5, 4, 24434398, 4],
    }
}

#[uniffi::export]
pub fn new_public_key_value(value: Vec<i32>) -> Value {
    Value::PublicKey { value }
}

// Test functions for 64-bit sequences
#[uniffi::export]
pub fn take_i64_list(value: Vec<i64>) -> Vec<i64> {
    value
}

#[uniffi::export]
pub fn take_u64_list(value: Vec<u64>) -> Vec<u64> {
    value
}

#[uniffi::export]
pub fn make_i64_list() -> Vec<i64> {
    vec![-9223372036854775808, 0, 9223372036854775807]
}

#[uniffi::export]
pub fn make_u64_list() -> Vec<u64> {
    vec![0, 12345678901234567890, 18446744073709551615]
}

// #[uniffi::export]
// pub fn new_map(value: HashMap<String, Value>) -> MapEntry {
//      todo!("Not done")
// }

#[uniffi::export]
pub fn take_value(value: Value) -> String {
    match value {
        Value::String { value } => format!("{}", value),
        Value::Bool { value } => format!("{}", value),
        Value::U8 { value } => format!("{}", value),
        Value::U16 { value } => format!("{}", value),
        Value::U32 { value } => format!("{}", value),
        Value::U64 { value } => format!("{}", value),
        Value::I8 { value } => format!("{}", value),
        Value::I16 { value } => format!("{}", value),
        Value::I32 { value } => format!("{}", value),
        Value::I64 { value } => format!("{}", value),
        Value::F32 { value } => format!("{}", value),
        Value::F64 { value } => format!("{}", value),
        //Value::Enum { discriminator, fields } => format!("{:?}, {:?}",discriminator, fields),
        // Value::NonHomogenousCollection { elements } => format!("{:?}", elements),
        // Value::HomogenousCollection { elements } => format!("{:?}", elements),
        // Value::Map { entries } => format!("{:?}", entries),
        Value::PublicKey { value } => format!("{:?}", value),
    }
}

#[derive(Debug, Clone, Enum)]
pub enum FlatEnum {
    One,
    Two,
    Three,
    Four,
}

// TODO: Add Collections (Maps, Vector, ...)
#[derive(Debug, Clone, Enum)]
pub enum Value {
    String { value: String },
    Bool { value: bool },
    U8 { value: u8 },
    U16 { value: u16 },
    U32 { value: u32 },
    U64 { value: u64 },

    I8 { value: i8 },
    I16 { value: i16 },
    I32 { value: i32 },
    I64 { value: i64 },
    F32 { value: f32 },
    F64 { value: f64 },
    // Enum {
    //     discriminator: u8,
    //     fields: Vec<Value>,
    // },
    // HomogenousCollection {
    //     elements: Vec<Value>,
    // },
    // Map {
    //     entries: Vec<MapEntry>,
    // },
    PublicKey { value: Vec<i32> },
}

#[derive(Clone, Debug, Record)]
pub struct MapEntry {
    pub key: Value,
    pub value: Value,
}

// Nested Collection Tests for Phase 2
#[uniffi::export]
fn take_nested_i32_list(value: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
    value
}

#[uniffi::export]
fn make_nested_i32_list() -> Vec<Vec<i32>> {
    vec![vec![1, 2, 3], vec![4, 5], vec![6, 7, 8, 9]]
}

#[uniffi::export]
fn take_optional_i32_list(value: Option<Vec<i32>>) -> Option<Vec<i32>> {
    value
}

#[uniffi::export]
fn make_optional_i32_list() -> Option<Vec<i32>> {
    Some(vec![10, 20, 30])
}

// HashMap/Map Tests for Phase 2.3
#[uniffi::export]
fn take_string_map(
    map: std::collections::HashMap<String, String>,
) -> std::collections::HashMap<String, String> {
    map
}

#[uniffi::export]
fn make_string_map() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert("hello".to_string(), "world".to_string());
    map.insert("foo".to_string(), "bar".to_string());
    map
}

#[uniffi::export]
fn take_int_map(
    map: std::collections::HashMap<i32, String>,
) -> std::collections::HashMap<i32, String> {
    map
}

#[uniffi::export]
fn make_int_map() -> std::collections::HashMap<i32, String> {
    let mut map = std::collections::HashMap::new();
    map.insert(1, "one".to_string());
    map.insert(2, "two".to_string());
    map.insert(42, "answer".to_string());
    map
}

// Complex Type Sequence Tests for Phase 2.2 - Debug step by step
#[uniffi::export]
fn take_simple_value_list(values: Vec<Value>) -> Vec<Value> {
    values
}

#[uniffi::export]
fn make_simple_value_list() -> Vec<Value> {
    vec![Value::String {
        value: "hello".to_string(),
    }]
}

// Test single MapEntry first (no sequence)
#[uniffi::export]
fn take_single_map_entry(entry: MapEntry) -> MapEntry {
    entry
}

#[uniffi::export]
fn make_single_map_entry() -> MapEntry {
    MapEntry {
        key: Value::String {
            value: "name".to_string(),
        },
        value: Value::String {
            value: "Alice".to_string(),
        },
    }
}

// Debug allocation size calculation
#[uniffi::export]
fn debug_string_value() -> Value {
    Value::String {
        value: "test".to_string(),
    }
}

// Test empty sequence first
#[uniffi::export]
fn take_empty_map_entry_list(entries: Vec<MapEntry>) -> Vec<MapEntry> {
    entries
}

#[uniffi::export]
fn make_empty_map_entry_list() -> Vec<MapEntry> {
    vec![]
}

// Test just simple integers first
#[uniffi::export]
fn take_i32_list_simple(values: Vec<i32>) -> Vec<i32> {
    values
}

#[uniffi::export]
fn make_i32_list_simple() -> Vec<i32> {
    vec![42]
}

// Test just enum sequences (we know these work)
#[uniffi::export]
fn take_bool_value_list(values: Vec<Value>) -> Vec<Value> {
    values
}

#[uniffi::export]
fn make_bool_value_list() -> Vec<Value> {
    vec![Value::Bool { value: true }]
}

// Now test simple record sequence (the problematic case)
#[uniffi::export]
fn take_simple_record_list(entries: Vec<MapEntry>) -> Vec<MapEntry> {
    entries
}

#[uniffi::export]
fn make_simple_record_list() -> Vec<MapEntry> {
    vec![MapEntry {
        key: Value::Bool { value: true },
        value: Value::Bool { value: false },
    }]
}

uniffi::include_scaffolding!("api");
