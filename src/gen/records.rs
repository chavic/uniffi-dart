use genco::prelude::*;
use uniffi_bindgen::backend::{CodeType, Literal};
use uniffi_bindgen::interface::{AsType, Record};

use super::oracle::{AsCodeType, DartCodeOracle};
use super::render::{Renderable, TypeHelperRenderer};
use super::types::generate_type;
use super::utils::{class_name, var_name};

#[derive(Debug)]
pub struct RecordCodeType {
    id: String,
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
        todo!("literal not implemented");
    }
}

impl Renderable for RecordCodeType {
    fn render_type_helper(&self, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
        if type_helper.check(&self.id) {
            quote!()
        } else if let Some(record_) = type_helper.get_record(&self.id) {
            generate_record(record_, type_helper)
        } else {
            todo!("render_type_helper not implemented");
        }
    }
}

pub fn generate_record(obj: &Record, type_helper: &dyn TypeHelperRenderer) -> dart::Tokens {
    let cls_name = &class_name(obj.name());
    let ffi_conv_name = &class_name(&obj.as_codetype().ffi_converter_name());
    quote! {
        class $cls_name {
            $(for f in obj.fields() => final $(generate_type(&f.as_type())) $(var_name(f.name()));)

            $(cls_name)._($(for f in obj.fields() => this.$(var_name(f.name())), ));
        }

        class $ffi_conv_name {

            static $cls_name lift(Api api, RustBuffer buf) {
                return $ffi_conv_name.read(api, buf.asUint8List()).value;
            }

            static LiftRetVal<$cls_name> read(Api api, Uint8List buf) {

                int new_offset = 0;

                $(for f in obj.fields() =>
                    final $(var_name(f.name()))_lifted = $(f.as_type().as_codetype().ffi_converter_name()).read(api, Uint8List.view(buf.buffer, new_offset));
                    final $(var_name(f.name())) = $(var_name(f.name()))_lifted.value;
                    new_offset += $(var_name(f.name()))_lifted.bytesRead;
                )
                return LiftRetVal($(cls_name)._(
                    $(for f in obj.fields() => $(var_name(f.name())),)
                ), new_offset);
            }

            static RustBuffer lowerIntoRustBuffer(Api api, $cls_name value) {
                throw "not yet supported";
            }
        }
    }
}
