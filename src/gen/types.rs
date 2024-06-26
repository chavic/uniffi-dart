use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
};

use genco::prelude::*;
use uniffi_bindgen::{
    interface::{FfiType, Type},
    ComponentInterface,
};

use super::{enums, functions, objects, primitives, records};
use super::{
    render::{AsRenderable, Renderer, TypeHelperRenderer},
    Config,
};

type FunctionDefinition = dart::Tokens;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ImportRequirement {
    Import { name: String },
    ImportAs { name: String, as_name: String },
}

// TODO: Handle importing external packages defined in the configuration.
// TODO: Finish refactor by moving all code that's not related to type helpers when Renderable has been implemented for the rest of the types
pub struct TypeHelpersRenderer<'a> {
    config: &'a Config,
    ci: &'a ComponentInterface,
    include_once_names: RefCell<HashMap<String, Type>>,
    imports: RefCell<BTreeSet<ImportRequirement>>,
}

#[allow(dead_code)]
impl<'a> TypeHelpersRenderer<'a> {
    pub fn new(config: &'a Config, ci: &'a ComponentInterface) -> Self {
        Self {
            config,
            ci,
            include_once_names: RefCell::new(HashMap::new()),
            imports: RefCell::new(BTreeSet::new()),
        }
    }

    pub fn external_type_package_name(&self, crate_name: &str) -> String {
        match self.config.external_packages.get(crate_name) {
            Some(name) => name.clone(),
            None => crate_name.to_string(),
        }
    }

    pub fn get_include_names(&self) -> HashMap<String, Type> {
        self.include_once_names.clone().into_inner()
    }
}

impl TypeHelperRenderer for TypeHelpersRenderer<'_> {
    // Checks if the type imports for each type have already been added
    fn include_once_check(&self, name: &str, ty: &Type) -> bool {
        let mut map = self.include_once_names.borrow_mut();
        let found = map.insert(name.to_string(), ty.clone()).is_some();
        drop(map);
        found
    }

    fn check(&self, name: &str) -> bool {
        let map = self.include_once_names.borrow();
        let contains = map.contains_key(&name.to_string());
        drop(map);
        contains
    }

    fn add_import(&self, name: &str) -> bool {
        self.imports.borrow_mut().insert(ImportRequirement::Import {
            name: name.to_owned(),
        })
    }

    fn add_import_as(&self, name: &str, as_name: &str) -> bool {
        self.imports
            .borrow_mut()
            .insert(ImportRequirement::ImportAs {
                name: name.to_owned(),
                as_name: as_name.to_owned(),
            })
    }

    fn get_object(&self, name: &str) -> Option<&uniffi_bindgen::interface::Object> {
        self.ci.get_object_definition(name)
    }

    fn get_enum(&self, name: &str) -> Option<&uniffi_bindgen::interface::Enum> {
        self.ci.get_enum_definition(name)
    }

    fn get_record(&self, name: &str) -> Option<&uniffi_bindgen::interface::Record> {
        self.ci.get_record_definition(name)
    }
}

impl Renderer<(FunctionDefinition, dart::Tokens)> for TypeHelpersRenderer<'_> {
    // TODO: Implimient a two pass system where the first pass will render the main code, and the second pass will render the helper code
    // this is so the generator knows what helper code to include.

    fn render(&self) -> (dart::Tokens, dart::Tokens) {
        // Render all the types and their helpers
        let types_definitions = quote! {
            $( for rec in self.ci.record_definitions() => $(records::generate_record(rec, self)))

            $( for enm in self.ci.enum_definitions() => $(enums::generate_enum(enm, self)))
            $( for obj in self.ci.object_definitions() => $(objects::generate_object(obj, self)))
        };

        // Render all the imports
        let imports: dart::Tokens = quote!();

        let function_definitions = quote!($( for fun in self.ci.function_definitions() => $(functions::generate_function("this", fun, self))));

        let helpers_definitions = quote! {
            $(for (_, ty) in self.get_include_names().iter() => $(ty.as_renderable().render_type_helper(self)) )
        };

        let types_helper_code = quote! {
            import "dart:async";
            import "dart:convert";
            import "dart:ffi";
            import "dart:io" show Platform, File, Directory;
            import "dart:isolate";
            import "dart:typed_data";
            import "package:ffi/ffi.dart";
            $(imports)

            class UniffiInternalError implements Exception {
                static const int bufferOverflow = 0;
                static const int incompleteData = 1;
                static const int unexpectedOptionalTag = 2;
                static const int unexpectedEnumCase = 3;
                static const int unexpectedNullPointer = 4;
                static const int unexpectedRustCallStatusCode = 5;
                static const int unexpectedRustCallError = 6;
                static const int unexpectedStaleHandle = 7;
                static const int rustPanic = 8;

                final int errorCode;
                final String? panicMessage;

                const UniffiInternalError(this.errorCode, this.panicMessage);

                static UniffiInternalError panicked(String message) {
                return UniffiInternalError(rustPanic, message);
                }

                @override
                String toString() {
                switch (errorCode) {
                    case bufferOverflow:
                    return "UniFfi::BufferOverflow";
                    case incompleteData:
                    return "UniFfi::IncompleteData";
                    case unexpectedOptionalTag:
                    return "UniFfi::UnexpectedOptionalTag";
                    case unexpectedEnumCase:
                    return "UniFfi::UnexpectedEnumCase";
                    case unexpectedNullPointer:
                    return "UniFfi::UnexpectedNullPointer";
                    case unexpectedRustCallStatusCode:
                    return "UniFfi::UnexpectedRustCallStatusCode";
                    case unexpectedRustCallError:
                    return "UniFfi::UnexpectedRustCallError";
                    case unexpectedStaleHandle:
                    return "UniFfi::UnexpectedStaleHandle";
                    case rustPanic:
                    return "UniFfi::rustPanic: $$panicMessage";
                    default:
                    return "UniFfi::UnknownError: $$errorCode";
                }
                }
            }

            const int CALL_SUCCESS = 0;
            const int CALL_ERROR = 1;
            const int CALL_PANIC = 2;

            class LiftRetVal<T> {
                final T value;
                final int bytesRead;
                const LiftRetVal(this.value, this.bytesRead);

                LiftRetVal<T> copyWithOffset(int offset) {
                    return LiftRetVal(value, bytesRead + offset);
                }
            }


            class RustCallStatus extends Struct {
                @Int8()
                external int code;
                external RustBuffer errorBuf;

                static Pointer<RustCallStatus> allocate({int count = 1}) =>
                calloc<RustCallStatus>(count * sizeOf<RustCallStatus>()).cast();
            }

            T noop<T>(T t) {
                return t;
            }

            T rustCall<T>(Api api, T Function(Pointer<RustCallStatus>) callback) {
                var callStatus = RustCallStatus.allocate();
                final returnValue = callback(callStatus);

                switch (callStatus.ref.code) {
                case CALL_SUCCESS:
                    calloc.free(callStatus);
                    return returnValue;
                case CALL_ERROR:
                    throw callStatus.ref.errorBuf;
                case CALL_PANIC:
                    if (callStatus.ref.errorBuf.len > 0) {
                        final message = utf8.decoder.convert(callStatus.ref.errorBuf.asUint8List());
                        calloc.free(callStatus);
                        throw UniffiInternalError.panicked(message);
                    } else {
                        calloc.free(callStatus);
                        throw UniffiInternalError.panicked("Rust panic");
                    }
                default:
                    throw UniffiInternalError(callStatus.ref.code, null);
                }
            }

            class RustBuffer extends Struct {
                @Uint64()
                external int capacity;

                @Uint64()
                external int len;

                external Pointer<Uint8> data;

                static RustBuffer fromBytes(Api api, ForeignBytes bytes) {
                    final _fromBytesPtr = api._lookup<
                    NativeFunction<
                        RustBuffer Function(ForeignBytes, Pointer<RustCallStatus>)>>($(format!("\"{}\"", self.ci.ffi_rustbuffer_from_bytes().name())));
                    final _fromBytes =
                    _fromBytesPtr.asFunction<RustBuffer Function(ForeignBytes, Pointer<RustCallStatus>)>();
                    return rustCall(api, (res) => _fromBytes(bytes, res));
                }

                // Needed so that the foreign language bindings can create buffers in which to pass complex data types across the FFI in the future
                static RustBuffer allocate(Api api, int size) {
                    final _allocatePtr = api._lookup<
                        NativeFunction<
                            RustBuffer Function(Int64, Pointer<RustCallStatus>)>>($(format!("\"{}\"", self.ci.ffi_rustbuffer_alloc().name())));
                    final _allocate = _allocatePtr.asFunction<RustBuffer Function(int, Pointer<RustCallStatus>)>();
                    return rustCall(api, (res) => _allocate(size, res));
                }

                void deallocate(Api api) {
                    final _freePtr = api._lookup<
                    NativeFunction<
                        Void Function(RustBuffer, Pointer<RustCallStatus>)>>($(format!("\"{}\"", self.ci.ffi_rustbuffer_free().name())));
                    final _free = _freePtr.asFunction<void Function(RustBuffer, Pointer<RustCallStatus>)>();
                    rustCall(api, (res) => _free(this, res));
                }

                Uint8List asUint8List() {
                    return data.cast<Uint8>().asTypedList(len);
                }

                @override
                String toString() {
                    return "RustBuffer { capacity: $capacity, len: $len, data: $data }";
                }
            }

            RustBuffer toRustBuffer(Api api, Uint8List data) {
                final length = data.length;

                final Pointer<Uint8> frameData = calloc<Uint8>(length); // Allocate a pointer large enough.
                final pointerList = frameData.asTypedList(length); // Create a list that uses our pointer and copy in the data.
                pointerList.setAll(0, data); // FIXME: can we remove this memcopy somehow?

                final bytes = calloc<ForeignBytes>();
                bytes.ref.len = length;
                bytes.ref.data = frameData;
                return RustBuffer.fromBytes(api, bytes.ref);
            }

            $(primitives::generate_wrapper_lifters())
            $(primitives::generate_wrapper_lowerers())

            class ForeignBytes extends Struct {
                @Int32()
                external int len;

                external Pointer<Uint8> data;
            }


            $(helpers_definitions)

            $(types_definitions)

        };

        (types_helper_code, function_definitions)
    }
}

pub fn generate_ffi_type(ret: Option<&FfiType>) -> dart::Tokens {
    let Some(ret_type) = ret else {
        return quote!(Void);
    };
    match *ret_type {
        FfiType::UInt8 => quote!(Uint8),
        FfiType::UInt16 => quote!(Uint16),
        FfiType::UInt32 => quote!(Uint32),
        FfiType::UInt64 => quote!(Uint64),
        FfiType::Int8 => quote!(Int8),
        FfiType::Int16 => quote!(Int16),
        FfiType::Int32 => quote!(Int32),
        FfiType::Int64 => quote!(Int64),
        FfiType::Float32 => quote!(Float),
        FfiType::Float64 => quote!(Double),
        FfiType::RustBuffer(ref inner) => match inner {
            Some(i) => quote!($i),
            _ => quote!(RustBuffer),
        },
        FfiType::RustArcPtr(_) => quote!(Pointer<Void>),
        _ => todo!("FfiType::{:?}", ret_type),
    }
}

pub fn generate_ffi_dart_type(ret: Option<&FfiType>) -> dart::Tokens {
    let Some(ret_type) = ret else {
        return quote!(void);
    };
    match *ret_type {
        FfiType::UInt8 => quote!(int),
        FfiType::UInt16 => quote!(int),
        FfiType::UInt32 => quote!(int),
        FfiType::UInt64 => quote!(int),
        FfiType::Int8 => quote!(int),
        FfiType::Int16 => quote!(int),
        FfiType::Int32 => quote!(int),
        FfiType::Int64 => quote!(int),
        FfiType::Float32 | FfiType::Float64 => quote!(double),
        FfiType::RustBuffer(ref inner) => match inner {
            Some(i) => quote!($i),
            _ => quote!(RustBuffer),
        },
        FfiType::RustArcPtr(_) => quote!(Pointer<Void>),
        _ => todo!("FfiType::{:?}", ret_type),
    }
}

pub fn generate_type(ty: &Type) -> dart::Tokens {
    match ty {
        Type::UInt8
        | Type::UInt32
        | Type::Int8
        | Type::Int16
        | Type::Int64
        | Type::UInt16
        | Type::Int32
        | Type::Duration
        | Type::UInt64 => quote!(int),
        Type::Float32 | Type::Float64 => quote!(double),
        Type::String => quote!(String),
        Type::Object { name, .. } => quote!($name),
        Type::Boolean => quote!(bool),
        Type::Optional { inner_type } => quote!($(generate_type(inner_type))?),
        Type::Sequence { inner_type } => quote!(List<$(generate_type(inner_type))>),
        Type::Enum { name, .. } => quote!($name),
        // Type::Record { name,..  } => quote!($name),
        _ => todo!("Type::{:?}", ty), // AbiType::Num(ty) => self.generate_wrapped_num_type(*ty),
                                      // AbiType::Isize | AbiType::Usize => quote!(int),
                                      // AbiType::Bool => quote!(bool),
                                      // AbiType::RefStr | AbiType::String => quote!(String),
                                      // AbiType::RefSlice(ty) | AbiType::Vec(ty) => {
                                      //     quote!(List<#(self.generate_wrapped_num_type(*ty))>)
                                      // }
                                      // AbiType::Option(ty) => quote!(#(self.generate_type(ty))?),
                                      // AbiType::Result(ty) => self.generate_type(ty),
                                      // AbiType::Tuple(tuple) => match tuple.len() {
                                      //     0 => quote!(void),
                                      //     1 => self.generate_type(&tuple[0]),
                                      //     _ => quote!(List<dynamic>),
                                      // },
                                      // AbiType::RefObject(ty) | AbiType::Object(ty) => quote!(#ty),
                                      // AbiType::RefIter(ty) | AbiType::Iter(ty) => quote!(Iter<#(self.generate_type(ty))>),
                                      // AbiType::RefFuture(ty) | AbiType::Future(ty) => {
                                      //     quote!(Future<#(self.generate_type(ty))>)
                                      // }
                                      // AbiType::RefStream(ty) | AbiType::Stream(ty) => {
                                      //     quote!(Stream<#(self.generate_type(ty))>)
                                      // }
                                      // AbiType::Buffer(ty) => quote!(#(ffi_buffer_name_for(*ty))),
                                      // AbiType::List(ty) => quote!(#(format!("FfiList{}", ty))),
                                      // AbiType::RefEnum(ty) => quote!(#(ty)),
    }
}
