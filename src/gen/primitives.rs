use crate::gen::render::{Renderable, TypeHelperRenderer};
use genco::prelude::*;
use paste::paste;
use uniffi_bindgen::backend::{CodeType, Literal};
use uniffi_bindgen::interface::{Radix, Type};

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
            | Type::Float64 => num_str,
            _ => panic!("Unexpected literal: {} is not a number", num_str),
        }
    }

    match literal {
        Literal::Boolean(v) => format!("{}", v),
        Literal::String(s) => format!("\"{}\"", s),
        Literal::Int(i, radix, type_) => typed_number(
            type_,
            match radix {
                Radix::Octal => format!("{:#x}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
        ),
        Literal::UInt(i, radix, type_) => typed_number(
            type_,
            match radix {
                Radix::Octal => format!("{:#x}", i),
                Radix::Decimal => format!("{}", i),
                Radix::Hexadecimal => format!("{:#x}", i),
            },
        ),
        Literal::Float(string, type_) => typed_number(type_, string.clone()),

        _ => unreachable!("Literal"),
    }
}

macro_rules! impl_code_type_for_primitive {
    ($T:ty, $class_name:literal, $canonical_name:literal) => {
        paste! {
            #[derive(Debug)]
            pub struct $T;

            impl CodeType for $T  {
                fn type_label(&self,) -> String {
                    $class_name.into()
                }

                fn literal(&self, literal: &Literal) -> String {
                    render_literal(&literal)
                }

                fn canonical_name(&self,) -> String {
                    $canonical_name.into()
                }

                fn ffi_converter_name(&self) -> String {
                    format!("FfiConverter{}", self.canonical_name())
                }

                // The following must create an instance of the converter object
                fn lower(&self) -> String {
                    format!("{}().lower", self.ffi_converter_name())
                }

                fn write(&self) -> String {
                    format!("{}().write", self.ffi_converter_name())
                }

                fn lift(&self) -> String {
                    format!("{}().lift", self.ffi_converter_name())
                }

                fn read(&self) -> String {
                    format!("{}().read", self.ffi_converter_name())
                }
            }
        }
    };
}

macro_rules! impl_renderable_for_primitive {
    ($T:ty, $class_name:literal, $canonical_name:literal, $allocation_size:literal) => {
        impl Renderable for $T {
            fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                // TODO: Need to modify behavior to allow
                // if (type_helper.check($canonical_name)) {
                //     return quote!()
                // }
                // This method can be expanded to generate type helper methods if needed.
                let mut endian = (if $canonical_name.contains("Float") {
                    "Endian.little"
                } else {
                    "Endian.big"
                });
                let _final_uintlist = (if $canonical_name.contains("Float") {
                    String::from($canonical_name) + "List.fromList(buf.reversed.toList())"
                } else {
                    String::from($canonical_name) + "List.fromList(buf.toList())"
                });

                let cl_name =  format!("FfiConverter{}", $canonical_name);
                let data_type = &$canonical_name
                    .replace("UInt", "Uint")
                    .replace("Double", "Float");
                let type_signature = if data_type.contains("Float") {
                    "double"
                } else {
                    endian = "";
                    "int"
                };

                quote! {
                    class $cl_name extends FfiConverter<$type_signature, $type_signature> 
                    with FfiConverterPrimitive<$type_signature> {
                        @override
                        $type_signature read(ByteData buffer, int offset) => buffer.get$data_type(offset);

                        @override
                        int size([$type_signature value = $allocation_size]) {
                          return $allocation_size;
                        }

                        @override
                        void write($type_signature value, ByteData buffer, int offset) =>
                        buffer.set$data_type(offset, value);
                    }
                }
            }
        }
    };

    (BooleanCodeType) => {
        impl Renderable for BooleanCodeType {
            fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                // if (type_helper.check($canonical_name)) {
                //     return quote!()
                // }
                // This method can be expanded to generate type helper methods if needed.
                quote! {
                    class FfiConverterBool implements FfiConverter<bool, int> {
                        const FfiConverterBool();
        
                        @override
                        bool lift(int value) => value != 0;
        
                        @override
                        int lower(bool value) => value ? 1 : 0;
        
                        @override
                        bool read(ByteData buffer, int offset) => buffer.getInt8(offset) != 0;
        
                        @override
                        void write(bool value, ByteData buffer, int offset) {
                        buffer.setInt8(offset, lower(value));
                        }
        
                        @override
                        int size(value) => 1;
                    }
                }
            }
        }
    };

    (StringCodeType) => {
        impl Renderable for StringCodeType {
            fn render_type_helper(&self, _type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                // This method can be expanded to generate type helper methods if needed.
                quote! {
                    // if (type_helper.check($canonical_name)) {
                    //     return quote!()
                    // }
                    class FfiConverterString implements FfiConverter<String, RustBuffer> {
                        const FfiConverterString();
                        // TODO: Figure out why there's spooky behavior here, default should be four, will fix later
                        String lift(RustBuffer value, [int offset = 0]) {
                            try {
                                final data = value.asTypedList().buffer.asUint8List(offset);
                                return utf8.decoder.convert(data);
                            } finally {
                                value.free();
                            }
                        }
        
                        @override
                        RustBuffer lower(String value) {
                            final buffer = toRustBuffer(Utf8Encoder().convert(value)); // TODO: Fix the meme copies issue by first fixing read
                            return buffer;
                        }
        
                        @override
                        String read(ByteData buffer, int offset) {
                            // TODO! : Fix this, it shouldn't append the lenth to every string, please remove first four bytes later
                            final length = buffer.getInt32(offset);
                            final stringBytes = buffer.buffer.asUint8List(offset + 4, length);
                            return utf8.decoder.convert(stringBytes);
                        }
        
                        @override
                        void write(String value, ByteData buffer, int offset) {
                        final stringBytes = utf8.encode(value);
                        buffer.setInt32(offset, stringBytes.length);
                        buffer.buffer.asUint8List(offset + 4).setAll(0, stringBytes);
                        }
        
                        @override
                        int size(value) => 4 + utf8.encode(value).length;
                    }
                }
            }
        }
    };

    (BytesCodeType, $class_name:literal, $canonical_name:literal, $allocation_size:literal) => {
        impl Renderable for $T {
            fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                if (type_helper.check($canonical_name)) {
                    return quote!(); // Return an empty string to avoid code duplication
                }
                // TODO: implement bytes ffi methods
                quote! {
                    class BytesFfiConverter extends FfiConverter<$canonical_name, RustBuffer> {
                        @override
                        int lift(RustBuffer buf, [int offset = 0]) {
                            // final uint_list = buf.toIntList();
                            // return uint_list.buffer.asByteData().get$canonical_name(1);
                        }

                        @override
                        RustBuffer lower(int value) {
                            // final uint_list = Uint8List.fromList([value ? 1 : 0]);
                            // final byteData = ByteData.sublistView(buf);
                            // byteData.setInt16(0, value, Endian.little);
                            // return buf;
                        }

                        @override
                        int read(ByteBuffer buf) {
                        //     // So here's the deal, we have two choices, could use Uint8List or ByteBuffer, leaving this for later
                        //     // performance reasons
                        //   throw UnimplementedError("Should probably implement read now");
                        }

                        @override
                        int size([T value]) {
                        //   return $allocation_size; // 1 = 8bits//TODO: Add correct allocation size for bytes, change the arugment type
                        }

                        @override
                        void write(int value, ByteBuffer buf) {
                            // throw UnimplementedError("Should probably implement read now");
                        }
                    }
                }
            }
        }
    };
}

impl_code_type_for_primitive!(BooleanCodeType, "bool", "Bool");
impl_code_type_for_primitive!(StringCodeType, "String", "String");
impl_code_type_for_primitive!(BytesCodeType, "Uint8List", "Uint8List");
impl_code_type_for_primitive!(Int8CodeType, "int", "Int8");
impl_code_type_for_primitive!(Int16CodeType, "int", "Int16");
impl_code_type_for_primitive!(Int32CodeType, "int", "Int32");
impl_code_type_for_primitive!(Int64CodeType, "int", "Int64");
impl_code_type_for_primitive!(UInt8CodeType, "int", "UInt8");
impl_code_type_for_primitive!(UInt16CodeType, "int", "UInt16");
impl_code_type_for_primitive!(UInt32CodeType, "int", "UInt32");
impl_code_type_for_primitive!(UInt64CodeType, "int", "UInt64");
impl_code_type_for_primitive!(Float32CodeType, "double", "Double32");
impl_code_type_for_primitive!(Float64CodeType, "double", "Double64");

impl_renderable_for_primitive!(BooleanCodeType);
impl_renderable_for_primitive!(StringCodeType);
// TODO: implement BytesCodeType
// impl_renderable_for_primitive!(BytesCodeType, "Uint8List", "Uint8List", 1);
impl_renderable_for_primitive!(Int8CodeType, "int", "Int8", 1);
impl_renderable_for_primitive!(Int16CodeType, "int", "Int16", 2);
impl_renderable_for_primitive!(Int32CodeType, "int", "Int32", 4);
impl_renderable_for_primitive!(Int64CodeType, "int", "Int64", 8);
impl_renderable_for_primitive!(UInt8CodeType, "int", "UInt8", 1);
impl_renderable_for_primitive!(UInt16CodeType, "int", "UInt16", 2);
impl_renderable_for_primitive!(UInt32CodeType, "int", "UInt32", 4);
impl_renderable_for_primitive!(UInt64CodeType, "int", "UInt64", 8);
impl_renderable_for_primitive!(Float32CodeType, "double", "Double32", 4);
impl_renderable_for_primitive!(Float64CodeType, "double", "Double64", 8);

// Delete these later

// TODO: Create struct that implement Renderer/Renderable for primitives
// Implementations for primitive type helpers

// pub fn generate_primitives_lifters() -> dart::Tokens {
//     quote!{
//         int liftInt8OrUint8(Uint8List buf, [int offset = 1]) {
//             return buf.buffer.asByteData().getInt8(offset);
//         }

//         int liftInt16OrUint16(Uint8List buf, [int offset = 1]) {
//             return buf.buffer.asByteData().getInt16(offset);
//         }

//         int liftInt32OrUint32(Uint8List buf, [int offset = 1]) {
//             return buf.buffer.asByteData().getInt32(offset);
//         }

//         int liftInt64OrUint64(Uint8List buf, [int offset = 1]) {
//             return buf.buffer.asByteData().getInt64(offset);
//         }

//         double liftFloat32(Uint8List buf, [int offset = 1]) {
//             return buf.buffer.asByteData().getFloat32(offset);
//         }

//         double liftFloat64(Uint8List buf, [int offset = 1]) {
//             return buf.buffer.asByteData().getFloat64(offset);
//         }

//         bool liftBoolean(Uint8List buf, [int offset = 1]) {
//             return buf.sublist(offset)[0] == 1;
//         }
//     }
// }

// pub fn generate_primitives_lifters() -> dart::Tokens {
//     quote!{
//         int? liftInt8OrUint8(Uint8List buf, [int offset = 1]) {
//             return buf.isEmpty ? null : buf.buffer.asByteData().getInt8(offset);
//         }

//         int? liftInt16OrUint16(Uint8List buf, [int offset = 1]) {
//             return buf.isEmpty ? null : buf.buffer.asByteData().getInt16(offset);
//         }

//         int? liftInt32OrUint32(Uint8List buf, [int offset = 1]) {
//             return buf.isEmpty ? null : buf.buffer.asByteData().getInt32(offset);
//         }

//         int? liftInt64OrUint64(Uint8List buf, [int offset = 1]) {
//             return buf.isEmpty ? null : buf.buffer.asByteData().getInt64(offset);
//         }

//         double? liftFloat32(Uint8List buf, [int offset = 1]) {
//             if (!buf.isEmpty) {
//                 double res = buf.buffer.asByteData().getFloat32(offset);
//                 res = double.parse(res.toStringAsFixed(6)); // Could adjust this later...
//                 return res;
//             } else {
//                 return null;
//             }

//            // return buf.isEmpty ? null : buf.buffer.asByteData().getFloat32(offset);
//         }

//         double? liftFloat64(Uint8List buf, [int offset = 1]) {
//             return buf.isEmpty ? null : buf.buffer.asByteData().getFloat64(offset);
//         }

//         bool? liftBoolean(Uint8List buf, [int offset = 1]) {
//             return buf.isEmpty ? null : (buf.sublist(offset)[0] == 1 ? true : false);
//         }
//     }
// }

pub fn generate_wrapper_lifters() -> dart::Tokens {
    quote! {
        class DataOffset<T> {
            final T? data;
            final int offset;
            DataOffset(this.data, this.offset);
        }

        // Todo!: Make this guy handle varaible strings
        DataOffset<T> liftVaraibleLength<T>(
            Uint8List buf, T? Function(Uint8List) lifter,
            [int offset = 1]) {
            final length = buf.buffer.asByteData().getInt32(offset); // the length in Uint8
            final liftedData = lifter(buf.sublist(offset + 4));
            return DataOffset(liftedData, length);
        }

        List<T> liftSequence<T>(Uint8List buf, Function(Uint8List, [int offset]) lifter, [int element_byte_size = 1,int offset = 0]) {
            List<T> res = [];
            buf = buf.sublist(offset);
            final length = buf.buffer.asByteData().getInt32(0);
            buf = buf.sublist(4);

            final element_byte_size = (buf.length ~/ length);
            offset = 0;

            for (var i = 0; i < length; i++) {
                offset = element_byte_size * i; // Update the offset for the next loop
                final item = lifter(buf, offset);
                res.add(item);
            }

            return res;
        }
    }
}

// pub fn generate_primitives_lowerers() -> dart::Tokens {
//     quote! {
//         // TODO: implement lowerers for primitives
//         Uint8List createUint8ListFromInt(int value) {
//             int length = value.bitLength ~/ 8 + 1;

//             // Ensure the length is either 4 or 8
//             if (length != 4 && length != 8) {
//             length = (value < 0x100000000) ? 4 : 8;
//             }

//             Uint8List uint8List = Uint8List(length);

//             for (int i = length - 1; i >= 0; i--) {
//             uint8List[i] = value & 0xFF;
//             value >>= 8;
//             }

//             return uint8List;
//         }

//         Uint8List lowerUint8(int value) {
//             final buf = Uint8List(1);
//             final byteData = ByteData.sublistView(buf);
//             byteData.setUint8(0, value);
//             return Uint8List.fromList(buf.toList());
//         }

//         Uint8List lowerInt8(int value) {
//             final buf = Uint8List(1);
//             final byteData = ByteData.sublistView(buf);
//             byteData.setInt8(0, value);
//             return Uint8List.fromList(buf.toList());
//         }

//         Uint8List lowerUint16(int value) {
//             final buf = Uint8List(2);
//             final byteData = ByteData.sublistView(buf);
//             byteData.setUint16(0, value);
//             return Uint8List.fromList(buf.toList());
//         }

//         Uint8List lowerInt16(int value) {
//             final buf = Uint8List(2);
//             final byteData = ByteData.sublistView(buf);
//             byteData.setInt16(0, value);
//             return Uint8List.fromList(buf.toList());
//         }

//         Uint8List lowerFloat32(double value) {
//             final buf = Uint8List(4);
//             final byteData = ByteData.sublistView(buf);
//             byteData.setFloat32(0, value, Endian.little);
//             return Uint8List.fromList(buf.reversed.toList());
//         }

//         Uint8List lowerFloat64(double value) {
//             final buf = Uint8List(8);
//             final byteData = ByteData.sublistView(buf);
//             byteData.setFloat64(0, value, Endian.little);
//             return Uint8List.fromList(buf.reversed.toList());
//         }
//     }
// }

pub fn generate_wrapper_lowerers() -> dart::Tokens {
    quote! {
        Uint8List createUint8ListFromInt(int value) {
            int length = value.bitLength ~/ 8 + 1;

            // Ensure the length is either 4 or 8
            if (length != 4 && length != 8) {
            length = (value < 0x100000000) ? 4 : 8;
            }

            Uint8List uint8List = Uint8List(length);

            for (int i = length - 1; i >= 0; i--) {
            uint8List[i] = value & 0xFF;
            value >>= 8;
            }

            return uint8List;
        }

        Uint8List lowerVaraibleLength<T>(T input, Uint8List Function(T) lowerer) {
            final lowered = lowerer(input);
            final length = createUint8ListFromInt(lowered.length);
            Uint8List res = Uint8List(lowered.length + length.length);
            res.setAll(0, length);
            res.setAll(length.length, lowered);
            return res;
        }

        Uint8List lowerSequence<T, V>(List<T> input, Uint8List Function(V) lowerer, int element_byte_size) {
          int capacity = input.length * element_byte_size;
          Uint8List items = Uint8List(capacity + 4); // Four bytes for the length
          int offset = 0;

          // Set the length of the vec
          items.setAll(offset, createUint8ListFromInt(capacity));
          offset += 4;

          for (var i = 0; i < input.length; i++) {
            items.setRange(
                offset, offset + element_byte_size, lowerer(input[i] as V));
            offset += element_byte_size;
          }

          print("Items from sequence");
          print(items);
          return items;
        }
    }
}
