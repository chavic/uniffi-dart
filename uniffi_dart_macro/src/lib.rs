use proc_macro::TokenStream;
use quote::{format_ident, quote};
use stringcase::pascal_case;
use syn::parse::Parse;
use syn::{parse_macro_input, Ident, ItemFn, LitStr, Type};

struct StreamAttr {
    item_type: Type,
    runtime: Option<LitStr>,
}

impl Parse for StreamAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let item_type: Type = input.parse()?;
        let mut runtime = None;

        while input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;

            if input.is_empty() {
                break;
            }

            let ident: Ident = input.parse()?;

            if ident != "runtime" {
                return Err(syn::Error::new_spanned(ident, "expected `runtime`"));
            }

            input.parse::<syn::Token![=]>()?;
            if runtime.is_some() {
                return Err(syn::Error::new(ident.span(), "duplicate `runtime` argument"));
            }

            let value: LitStr = input.parse()?;
            runtime = Some(value);
        }

        Ok(StreamAttr { item_type, runtime })
    }
}

#[proc_macro_attribute]
pub fn export_stream(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as StreamAttr);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let vis = &input.vis;
    let struct_name = format_ident!("{}StreamExt", pascal_case(&fn_name.to_string()));
    let create_fn_name = format_ident!("create_stream_{}", fn_name);
    let StreamAttr { item_type, runtime } = attr;
    let runtime_attr = if let Some(runtime) = runtime {
        quote!(#[uniffi::export(async_runtime = #runtime)])
    } else {
        quote!(#[uniffi::export(async_runtime = "tokio")])
    };

    let expanded = quote! {
        #input

        #[derive(uniffi::Object)]
        #vis struct #struct_name {
            stream: std::sync::Mutex<std::pin::Pin<Box<dyn futures::Stream<Item = #item_type> + Send>>>,
        }

        #runtime_attr
        impl #struct_name {
            #[uniffi::constructor]
            pub fn new() -> std::sync::Arc<Self> {
                std::sync::Arc::new(Self {
                    stream: std::sync::Mutex::new(Box::pin(#fn_name())),
                })
            }

            pub async fn next(&self) -> Option<#item_type> {
                futures::future::poll_fn(|cx| {
                    let mut stream = self
                        .stream
                        .lock()
                        .expect("stream mutex poisoned");
                    stream.as_mut().poll_next(cx)
                }).await
            }

        }

        #[uniffi::export]
        #vis fn #create_fn_name() -> std::sync::Arc<#struct_name> {
            #struct_name::new()
        }
    };

    TokenStream::from(expanded)
}
