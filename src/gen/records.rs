use super::oracle::{AsCodeType, DartCodeOracle};
use super::render::{Renderable, TypeHelperRenderer};
use super::types::generate_type;
use super::utils::{class_name, var_name};
use genco::prelude::*;
use uniffi_bindgen::backend::{CodeType, Literal};
use uniffi_bindgen::interface::{AsType, Record};

#[derive(Debug)]
pub struct RecordCodeType {
    id: String,
    #[allow(dead_code)]
    module_path: String,
}

impl RecordCodeType {
    pub fn new(id: String, module_path: String) -> Self {
        Self { id, module_path }
    }
}

impl CodeType for RecordCodeType {
    fn type_label(&self) -> String {
        DartCodeOracle::class_name(&self.id)
    }

    fn canonical_name(&self) -> String {
        self.id.to_string()
    }

    fn literal(&self, _literal: &Literal) -> String {
        unreachable!();
    }
}

impl Renderable for RecordCodeType {
    // Semantically, it may make sense to render object here, but we don't have enough information. So we render it with help from type_helper
    fn render(&self) -> dart::Tokens {
        quote!()
    }

    fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        if type_helper.check(&self.id) {
            quote!()
        } else if let Some(obj) = type_helper.get_record(&self.id) {
            generate_record(obj, type_helper)
        } else {
            unreachable!()
        }
    }
}

pub fn generate_record(obj: &Record, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
    let cls_name = &DartCodeOracle::class_name(obj.name());
    let ffi_conv_name = &DartCodeOracle::class_name(&obj.as_codetype().ffi_converter_name());
    for f in obj.fields() {
        // make sure all our field types are added to the includes
        type_helper.include_once_check(&f.as_codetype().canonical_name(), &f.as_type());
    }

    quote! {

            class $cls_name {
                $(for f in obj.fields() => final $(generate_type(&f.as_type())) $(var_name(f.name()));)

                $(cls_name)._($(for f in obj.fields() => this.$(var_name(f.name())), ));
            }


            class $ffi_conv_name {

                static $cls_name lift(RustBuffer buf) {
                    return $ffi_conv_name.read(buf.asTypedList().buffer.asByteData());
                }

                static $cls_name read(ByteData buffer, [int offset = 0]) {
                    int new_offset = offset;
                    $(for f in obj.fields() =>
                        final $(var_name(f.name())) = $(f.as_type().as_codetype().ffi_converter_name()).read(buffer, new_offset);
                        new_offset += $(f.as_type().as_codetype().ffi_converter_name()).size($(var_name(f.name())));
                    )
                    return $(cls_name)._(
                        $(for f in obj.fields() => $(var_name(f.name())),)
                    );
                }

                static RustBuffer lower($cls_name value) {
                    final total_length = $(for f in obj.fields() => $(f.as_type().as_codetype().ffi_converter_name()).size(value.$(var_name(f.name()))) + ) 0;
                    final buf = Uint8List(total_length);
                    $ffi_conv_name.write(api, value, buf);
                    return toRustBuffer(api, buf);
                }

                static int write($cls_name value, ByteData buffer, int offset) {
                    int new_offset = buf.offsetInBytes;

                    $(for f in obj.fields() =>
                    new_offset += $(f.as_type().as_codetype().ffi_converter_name()).write(api, value.$(var_name(f.name())), Uint8List.view(buf.buffer, new_offset));
                    )
                    return new_offset - buf.offsetInBytes;
                }
            }
        }
    }


