use genco::lang::dart;
use genco::prelude::*;
use paste::paste;
use uniffi_bindgen::backend::CodeType;
use uniffi_bindgen::interface::{Literal, Type};

use super::oracle::{AsCodeType, DartCodeOracle};
use crate::gen::render::{Renderable, TypeHelperRenderer};

fn render_literal(literal: &Literal, inner: &Type) -> String {
    match literal {
        Literal::None => "null".into(),
        Literal::EmptySequence => "[]".into(),
        Literal::EmptyMap => "{}".into(),

        // For optionals
        _ => DartCodeOracle::find(inner).literal(literal),
    }
}

macro_rules! impl_code_type_for_compound {
     ($T:ty, $type_label_pattern:literal, $canonical_name_pattern: literal) => {
        paste! {
            #[derive(Debug)]
            pub struct $T {
                self_type: Type,
                inner: Type,
            }

            impl $T {
                pub fn new(self_type: Type, inner: Type) -> Self {
                    Self { self_type, inner }
                }
                fn inner(&self) -> &Type {
                    &self.inner
                }
            }

            impl CodeType for $T  {
                fn type_label(&self) -> String {
                    format!($type_label_pattern, DartCodeOracle::find(self.inner()).type_label())
                }

                fn canonical_name(&self) -> String {
                    format!($canonical_name_pattern, DartCodeOracle::find(self.inner()).canonical_name())
                }

                fn literal(&self, literal: &Literal) -> String {
                    render_literal(literal, self.inner())
                }

                fn ffi_converter_name(&self) -> String {
                    format!("FfiConverter{}", self.canonical_name())
                }

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
    }
 }

macro_rules! impl_renderable_for_compound {
    ($T:ty, $type_label_pattern:literal, $canonical_name_pattern: literal) => {
       paste! {
            impl Renderable for $T {
                fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                    type_helper.include_once_check($canonical_name_pattern, &self.self_type);
                    let inner_codetype = DartCodeOracle::find(self.inner());
                    // let inner_type_label = inner_codetype.type_label();

                    type_helper.include_once_check(&inner_codetype.canonical_name(), &self.inner()); // Add the Inner FFI Converter

                    let cl_name = format!("{}{}","FfiConverter",  format!($canonical_name_pattern, inner_codetype.canonical_name()));
                    
                    // let type_label = &format!($type_label_pattern, &inner_type_label);
                    let type_label = &self.type_label();

                    let (lift_fn, lower_fn) = if cl_name.contains("Bool") {
                        ("FfiConverterBool().lift(intlist[5])".to_string(), "FfiConverterBool().lower(value) == true".to_string())
                    } else if cl_name.contains("String") {
                        // Only pass the string data to the lifter
                        // FfiConverterInt16().lift(FfiConverterInt16().read(intlist.buffer.asByteData(), offset))
                        (inner_codetype.lift() + "(buf, 5)" , String::from("value"))
                    } else {
                        (
                            format!("{}{}{}{}{}",  &inner_codetype.lift(), "(", &inner_codetype.read(), "(intlist.buffer.asByteData(), offset", "))"),
                            self.inner().as_codetype().lower() + "(value)",
                            
                        )
                    };


                    let inner_cl_converter_name = &inner_codetype.ffi_converter_name();
                    let inner_data_type = &inner_codetype.canonical_name().replace("UInt", "Uint").replace("Double", "Float");
                    let _inner_type_signature = if inner_data_type.contains("Float") { "double" } else { "int" };


                    quote! {
                        class $cl_name extends FfiConverter<$type_label, RustBuffer> {
                            @override
                            $type_label lift(RustBuffer buf, [int offset = 1]) {
                                var intlist = buf.toIntList();
                                if (intlist.isEmpty || intlist.first == 0){
                                    return null;
                                }
                                return $lift_fn;
                            }

                            @override
                            RustBuffer lower($type_label value) {
                                if (value == null) {
                                    final res = Uint8List(1);
                                    res.first = 0;
                                    return toRustBuffer(res);
                                }      
                                // converting the inner
                                final inner = $lower_fn;
                                // preparing the outer
                                final offset = 1;
                                final res = Uint8List($inner_cl_converter_name().size(value) + offset);
                                $inner_cl_converter_name().write(inner, res.buffer.asByteData(), offset); // Sets the data starting at the offset
                                res.first = 1;

                                return toRustBuffer(res);
                            }

                            @override
                            $type_label read(ByteData buf, int offset) {
                                // So here's the deal, we have two choices, could use Uint8List or ByteBuffer, leaving this for later
                                // considerations, after research on performance implications
                                throw UnimplementedError("Should probably implement read now");
                            }

                            @override
                            int size([$type_label value]) {
                                return $inner_cl_converter_name().size(value!) + 4;
                            }

                            @override
                            void write($type_label value, ByteData buffer, int offset) {
                                throw UnimplementedError("Should probably implement writes now");
                            }
                        }
                    }
                }
            }
       }
   };

   (SequenceCodeType, $canonical_name_pattern: literal) => {
        paste! {
            impl Renderable for SequenceCodeType {
                fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
                    type_helper.include_once_check($canonical_name_pattern, &self.self_type);
                    let inner_codetype = DartCodeOracle::find(self.inner());
                    let inner_type_label = &inner_codetype.type_label();

                    type_helper.include_once_check(&inner_codetype.canonical_name(), &self.inner()); // Add the Inner FFI Converter

                    let cl_name = format!("{}{}","FfiConverter",  format!($canonical_name_pattern, inner_codetype.canonical_name()));
                    let type_label = &format!("List<{}>", &inner_type_label);

                    let inner_cl_converter_name = &self.inner().as_codetype().ffi_converter_name();
                    let inner_data_type = &inner_codetype.canonical_name().replace("UInt", "Uint").replace("Double", "Float");
                    let _inner_type_signature = if inner_data_type.contains("Float") { "double" } else { "int" };
                    // TODO: Generate the proper lifter for each of the items

                    let (lift_fn, lower_fn) = if cl_name.contains("Bool") {
                        ("BoolFfiConverter().lift(intlist[offset])".to_string(), "Uint8List.fromList([BoolFfiConverter().lower(value[i])])".to_string())
                    } else if cl_name.contains("String") {
                        // Only pass the string data to the lifter
                        (inner_codetype.read() + "(buf.asTypedList().buffer.asByteData(), offset + 5)" , inner_codetype.lower() + "(value[i])")
                    } else {
                        (inner_codetype.read() + "(buf.asTypedList().buffer.asByteData(), offset)" ,  inner_codetype.lower() + "(value[i])")
                    };
                    let allocation_fn_expr = &format!("{}{}", inner_cl_converter_name.to_owned(), "().size(item)");
                    let write_fn_expr = &format!("{}{}", inner_cl_converter_name.to_owned(), "().write(item, item_intlist.buffer.asByteData(), 0)");

                    // We may need to rethink how we redo this, but it's largly due to the fact that macros_rules! confuse qoute!, try removing it, see what happens.
                    let lift_fn_clone = lift_fn.clone();
                    let lower_fn_clone = lower_fn.clone();

                    quote! {
                        class $cl_name extends FfiConverter<$type_label, RustBuffer> {
                            @override
                            $type_label lift(RustBuffer buf, [int offset = 0]) {
                                $type_label res = [];
                                var intlist = buf.toIntList();
                                final length = intlist.buffer.asByteData().getInt32(offset);
                                offset += 4;
                                intlist = intlist.sublist(offset);


                                for (var i = 0; i < length; i++) {
                                    final item = $lift_fn;
                                    offset += $allocation_fn_expr;
                                    res.add(item);
                                }

                                return res;
                            }

                            @override
                            RustBuffer lower($type_label value) {
                                List<Uint8List> items = [createUint8ListFromInt(value.length)];

                                for (var i = 0; i < value.length; i++) {
                                    // int item = FfiConverterInt32().lower(value[i]);
                                    // int item_size = FfiConverterInt32().size(item);
                                    // Uint8List item_intlist = Uint8List(item_size);
                                    // FfiConverterInt32().write(item, item_intlist.buffer.asByteData(), 0);
                                    // items.add(item_intlist);
                                    $inner_type_label item = $lower_fn;
                                    int item_size = $allocation_fn_expr;
                                    Uint8List item_intlist = Uint8List(item_size);
                                    $write_fn_expr;
                                    items.add(item_intlist);
                                }

                                Uint8List uint_list = Uint8List.fromList(items.expand((inner) => inner).toList());

                                return toRustBuffer(uint_list);
                            }

                            @override
                            $type_label read(ByteData buf, int offset) {
                                throw UnimplementedError("Should probably implement writes now");
                            }

                            @override
                            int size([$type_label? value]) {
                                // TODO: Change allocation size to use the first 4 bits of the list given
                                return ($inner_cl_converter_name().size() * value!.length) + 4;
                            }

                            @override
                            void write($type_label value, ByteData buf, int offset) {
                                List<Uint8List> items = [createUint8ListFromInt(value.length)];

                                for (var i = 0; i < value.length; i++) {
                                    
                                    $inner_type_label item = $lower_fn_clone;
                                    int item_size = $allocation_fn_expr;
                                    Uint8List item_intlist = Uint8List(item_size);
                                    $write_fn_expr;
                                    items.add(item_intlist);
                                }

                                Uint8List uint_list = Uint8List.fromList(items.expand((inner) => inner).toList());

                                buf.buffer.asInt8List().setAll(0, uint_list);
                            }
                        }
                    }
                }
            }
        }
   }
}

impl_code_type_for_compound!(OptionalCodeType, "{}?", "Optional{}");
impl_code_type_for_compound!(SequenceCodeType, "List<{}>", "Sequence{}");

impl_renderable_for_compound!(OptionalCodeType, "{}?", "Optional{}");
impl_renderable_for_compound!(SequenceCodeType, "Sequence{}");
