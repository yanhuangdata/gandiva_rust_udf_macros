mod udf_macro_test;
mod quote_helper;
mod attr_parser;

extern crate proc_macro;

use crate::attr_parser::{extract_params, extract_udf_meta};
use crate::quote_helper::{
    function_wrapper_quote, is_returning_string, load_registered_udfs_quote, process_arg,
    register_func_meta_quote, string_function_wrapper_quote,
};
use quote::{format_ident, quote};
use syn::{FnArg, ReturnType};
use gandiva_rust_udf_common::map_type;
use crate::quote_helper::{is_returning_string, string_function_wrapper_quote, function_wrapper_quote,
                          register_func_meta_quote, process_arg, load_registered_udfs_quote};
use crate::attr_parser::{extract_needs_context, extract_params};

#[proc_macro_attribute]
pub fn udf_registry(
    _attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    udf_registry_impl(input).into()
}

fn udf_impl(
    input: proc_macro2::TokenStream,
    name: Option<String>,
    aliases: Vec<String>,
    needs_context: bool,
    result_nullable: Option<String>,
) -> proc_macro2::TokenStream {
    let function = extract_params(input);
    let function_name = &function.sig.ident;
    let return_type = &function.sig.output;

    let mut wrapper_args = Vec::new();
    let mut call_args = Vec::new();
    let mut arg_types = Vec::new();
    let is_returning_string = is_returning_string(return_type);
    let final_needs_context = needs_context || is_returning_string;

    if final_needs_context {
        wrapper_args.push(quote! { ctx: i64 });
    }

    for input in &function.sig.inputs {
        match input {
            FnArg::Typed(pat_type) => {
                process_arg(pat_type, &mut wrapper_args, &mut call_args, &mut arg_types);
            }
            _ => {
                panic!("Unsupported function argument type");
            }
        }
    }

    let wrapper_name = format_ident!("{}_{}", function_name, arg_types.join("_"));

    let expanded = match return_type {
        ReturnType::Default => {
            panic!("The function to be wrapped must have a return type.");
        }
        ReturnType::Type(_, ty) => {
            // if return type is String, use gandiva context function to allocate memory
            let return_type_str = quote!(#ty).to_string();
            let mut return_arrow_type = map_type(return_type_str.as_str());
            let wrapper_func = if return_type_str == "String" {
                return_arrow_type = "utf8".to_string();
                wrapper_args.push(quote! { out_len: *mut i32 });
                string_function_wrapper_quote(
                    &function,
                    &wrapper_name,
                    &mut wrapper_args,
                    &function_name,
                    &mut call_args,
                )
            } else {
                function_wrapper_quote(
                    &function,
                    &wrapper_name,
                    &mut wrapper_args,
                    &function_name,
                    &mut call_args,
                    ty,
                )
            };
            let register_func_meta = register_func_meta_quote(
                &function_name,
                &arg_types,
                &wrapper_name,
                name,
                aliases,
                final_needs_context,
                result_nullable,
                &return_arrow_type,
            );
            quote! {
                #wrapper_func
                #register_func_meta
            }
        }
    };
    expanded
}

#[proc_macro_attribute]
pub fn udf(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match extract_udf_meta(attrs.into()) {
        Ok((name, aliases, needs_context, result_nullable)) => {
            let input = proc_macro2::TokenStream::from(input);
            udf_impl(input, name, aliases, needs_context, result_nullable).into()
        }
        Err(e) => syn::Error::new_spanned(e.to_compile_error(), e.to_string())
            .to_compile_error()
            .into(),
    }
}

fn udf_registry_impl(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let function = extract_params(input);
    load_registered_udfs_quote(function).into()
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[proc_macro]
pub fn get_version(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    std::format!("fn get_macro_version() -> String {{ \"{}\".to_string() }}", VERSION).parse().unwrap()
}
