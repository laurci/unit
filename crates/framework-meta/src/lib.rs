use darling::{export::NestedMeta, Error, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, FnArg, PatIdent, PatType, Result,
};
use unit_abi::header::{encode_abi_header, AbiHeader};

struct ApplicationMacroInput {
    name: syn::LitStr,
}

impl Parse for ApplicationMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        /* Example
         * application! {
         *    name = "demo",
         * }
         */

        let _name_ident: syn::Ident = input.parse()?;
        let _eq: syn::Token![=] = input.parse()?;
        let name_lit: syn::LitStr = input.parse()?;

        let _comma: syn::Token![,] = input.parse()?;

        Ok(ApplicationMacroInput { name: name_lit })
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn application(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ApplicationMacroInput);

    let Ok(magic_bytes) = encode_abi_header(&AbiHeader {
        name: item.name.value(),
    }) else {
        abort!(Span::call_site(), "Failed to encode ABI header");
    };

    let magic_header = syn::LitByteStr::new(&magic_bytes, Span::call_site());

    quote! {
        #[no_mangle]
        static UNIT_MAGIC_HEADER: &'static [u8] = #magic_header;

        unit::data! { pub runtime: unit::tokio::runtime::Runtime = unit::tokio::runtime::Builder::new_current_thread().build().unwrap() }

        fn main() {}

        #[no_mangle]
        pub extern "C" fn unit_alloc_bytes(len: i32) -> *mut u8 {
            let mut v = Vec::with_capacity(len as usize);
            let ptr = v.as_mut_ptr();
            std::mem::forget(v);
            return ptr;
        }

        #[no_mangle]
        pub extern "C" fn unit_free_bytes(ptr: *mut u8, len: i32) {
            unsafe {
                let _ = Vec::from_raw_parts(ptr, len as usize, len as usize);
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn init(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let item_fn_name = &item_fn.sig.ident;

    quote! {
        #[no_mangle]
        pub extern "C" fn unit_init() -> i32 {
            crate::runtime().block_on(#item_fn_name());

            return 0;
        }

        #item_fn
    }
    .into()
}

#[proc_macro_attribute]
pub fn cleanup(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let item_fn_name = &item_fn.sig.ident;

    quote! {
        #[no_mangle]
        pub extern "C" fn unit_cleanup() {
            crate::runtime().block_on(#item_fn_name());
        }

        #item_fn
    }
    .into()
}

#[proc_macro_attribute]
pub fn message(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let item_fn_name = &item_fn.sig.ident;

    quote! {
        #[no_mangle]
        pub extern "C" fn unit_message(ptr: i32, len: u32) -> i32 {
            let data = unsafe {
                let slice = ::std::slice::from_raw_parts(ptr as _, len as _);
                unit::proto::decode_runtime_proto_message::<unit::proto::WsMessage>(slice.to_vec()).unwrap()
            };

            crate::runtime().block_on(#item_fn_name(&data));

            return 0;
        }

        #item_fn
    }
    .into()
}

#[derive(Debug, FromMeta)]
struct TopicArgs {
    name: String,
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn topic(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };

    let item_fn = syn::parse_macro_input!(item as syn::ItemFn);
    let item_stmts = item_fn.block.stmts;

    if item_fn.sig.inputs.len() != 1 {
        abort! {
            item_fn.sig.inputs,
            "Topic function must have only one argument."
        }
    }

    let first_arg = &item_fn.sig.inputs[0];
    let first_arg_name = match first_arg {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            syn::Pat::Ident(PatIdent { ident, .. }) => ident,
            _ => abort! {
                pat,
                "Topic function argument must be a named identifier."
            },
        },
        _ => abort! {
            first_arg,
            "Topic function argument must be a named identifier."
        },
    };

    // TODO: FromEvent trait
    // let first_arg_type = match first_arg {
    //     FnArg::Typed(PatType { ty, .. }) => match &**ty {
    //         syn::Type::Reference(TypeReference { elem, .. }) => match &**elem {
    //             syn::Type::Path(TypePath { path, .. }) => path,
    //             _ => abort! {
    //                 ty,
    //                 "Topic function argument must be a struct that implementes FromEvent<T>."
    //             },
    //         },
    //         syn::Type::Path(TypePath { path, .. }) => path,
    //         _ => abort! {
    //             ty,
    //             "Topic function argument must be a struct that implementes FromEvent<T>."
    //         },
    //     },
    //     _ => abort! {
    //         first_arg,
    //         "Topic function argument must be a struct that implementes FromEvent<T>."
    //     },
    // };

    let args = match TopicArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };

    let topic_name = args.name;
    let extern_fn_name = format!("unit_topic_{}", topic_name.to_lowercase().replace("-", "_"));
    let extern_fn_name = Ident::new(&extern_fn_name, Span::call_site());

    quote! {
        #[no_mangle]
        pub extern "C" fn #extern_fn_name(ptr: i32, len: u32) -> i32 {
            let event = unsafe {
                let slice = ::std::slice::from_raw_parts(ptr as _, len as _);
                unit::proto::decode_runtime_proto_message::<unit::proto::CrossbarMessage>(slice.to_vec()).unwrap()
            };

            let #first_arg_name = event.content;

            crate::runtime().block_on(async move {
                #(#item_stmts)*
            });

            return 0;
        }
    }
    .into()
}
