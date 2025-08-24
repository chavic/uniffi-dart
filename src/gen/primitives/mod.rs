#[macro_use]
mod macros;
mod boolean;
mod duration;
mod string;

use crate::gen::render::{Renderable, TypeHelperRenderer};
use crate::gen::CodeType;
use genco::prelude::*;
use paste::paste;
use uniffi_bindgen::backend::Literal;
use uniffi_bindgen::interface::{Radix, Type};

pub use boolean::BooleanCodeType;
pub use duration::DurationCodeType;
pub use string::StringCodeType;

fn render_literal(literal: &Literal) -> String {
    fn typed_number(type_: &Type, num_str: String) -> String {
        match type_ {
            Type::Int8
            | Type::UInt8
            | Type::Int16
            | Type::UInt16
            | Type::Int32
            | Type::UInt32
            | Type::UInt64
            | Type::Float32
            | Type::Float64
            | Type::Duration => num_str,
            _ => panic!("Unexpected literal: {num_str} is not a number"),
        }
    }

    match literal {
        Literal::Boolean(v) => format!("{v}"),
        Literal::String(s) => format!("'{s}'"),
        Literal::Int(i, radix, type_) => typed_number(
            type_,
            match radix {
                Radix::Octal => format!("{i:#x}"),
                Radix::Decimal => format!("{i}"),
                Radix::Hexadecimal => format!("{i:#x}"),
            },
        ),
        Literal::UInt(i, radix, type_) => typed_number(
            type_,
            match radix {
                Radix::Octal => format!("{i:#x}"),
                Radix::Decimal => format!("{i}"),
                Radix::Hexadecimal => format!("{i:#x}"),
            },
        ),
        Literal::Float(string, type_) => typed_number(type_, string.clone()),
        _ => unreachable!("Literal"),
    }
}

impl_code_type_for_primitive!(BytesCodeType, "Uint8List", "Uint8List");
impl_code_type_for_primitive!(Int8CodeType, "int", "Int8");
impl_code_type_for_primitive!(Int16CodeType, "int", "Int16");
impl_code_type_for_primitive!(Int32CodeType, "int", "Int32");
impl_code_type_for_primitive!(Int64CodeType, "BigInt", "Int64");
impl_code_type_for_primitive!(UInt8CodeType, "int", "UInt8");
impl_code_type_for_primitive!(UInt16CodeType, "int", "UInt16");
impl_code_type_for_primitive!(UInt32CodeType, "int", "UInt32");
impl_code_type_for_primitive!(UInt64CodeType, "BigInt", "UInt64");
impl_code_type_for_primitive!(Float32CodeType, "double", "Double32");
impl_code_type_for_primitive!(Float64CodeType, "double", "Double64");

impl_renderable_for_primitive!(BytesCodeType, "Uint8List", "Uint8List");
impl_renderable_for_primitive!(Int8CodeType, "int", "Int8", 1);
impl_renderable_for_primitive!(Int16CodeType, "int", "Int16", 2);
impl_renderable_for_primitive!(Int32CodeType, "int", "Int32", 4);
impl_renderable_for_primitive!(UInt8CodeType, "int", "UInt8", 1);
impl_renderable_for_primitive!(UInt16CodeType, "int", "UInt16", 2);
impl_renderable_for_primitive!(UInt32CodeType, "int", "UInt32", 4);
impl_renderable_for_primitive!(Float32CodeType, "double", "Double32", 4);
impl_renderable_for_primitive!(Float64CodeType, "double", "Double64", 8);

// Custom implementations for 64-bit integer types that use BigInt
impl Renderable for Int64CodeType {
    fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        let cl_name = &self.ffi_converter_name();
        let type_signature = &self.type_label();

        quote! {
            class $cl_name {
                static $type_signature lift(int value) {
                    return BigInt.from(value);
                }

                static LiftRetVal<$type_signature> read(Uint8List buf) {
                    final value = buf.buffer.asByteData(buf.offsetInBytes).getInt64(0);
                    return LiftRetVal(BigInt.from(value), 8);
                }

                static int lower($type_signature value) {
                    // Convert BigInt to int, checking bounds
                    if (value < BigInt.from(-9223372036854775808) || value > BigInt.from(9223372036854775807)) {
                        throw ArgumentError("Value out of range for i64: " + value.toString());
                    }
                    return value.toInt();
                }

                static int allocationSize([$type_signature? value]) {
                    return 8;
                }

                static int write($type_signature value, Uint8List buf) {
                    buf.buffer.asByteData(buf.offsetInBytes).setInt64(0, lower(value));
                    return 8;
                }
            }
        }
    }
}

impl Renderable for UInt64CodeType {
    fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        let cl_name = &self.ffi_converter_name();
        let type_signature = &self.type_label();

        quote! {
            class $cl_name {
                static $type_signature lift(int value) {
                    // Convert unsigned int64 to BigInt, handling the unsigned nature
                    if (value < 0) {
                        // Handle unsigned interpretation of negative signed value
                        return BigInt.from(value) + (BigInt.one << 64);
                    }
                    return BigInt.from(value);
                }

                static LiftRetVal<$type_signature> read(Uint8List buf) {
                    final value = buf.buffer.asByteData(buf.offsetInBytes).getUint64(0);
                    return LiftRetVal(BigInt.from(value), 8);
                }

                static int lower($type_signature value) {
                    // Convert BigInt to int, checking bounds for u64
                    if (value < BigInt.zero || value > BigInt.parse("18446744073709551615")) {
                        throw ArgumentError("Value out of range for u64: " + value.toString());
                    }
                    // Handle conversion to signed representation if needed
                    if (value > BigInt.from(9223372036854775807)) {
                        return (value - (BigInt.one << 64)).toInt();
                    }
                    return value.toInt();
                }

                static int allocationSize([$type_signature? value]) {
                    return 8;
                }

                static int write($type_signature value, Uint8List buf) {
                    buf.buffer.asByteData(buf.offsetInBytes).setUint64(0, lower(value));
                    return 8;
                }
            }
        }
    }
}
