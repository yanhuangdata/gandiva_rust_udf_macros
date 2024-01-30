mod udf_macro_test;
mod type_mapping;
mod quote_helper;
mod attr_parser;

extern crate proc_macro;

use quote::{format_ident, quote};
use syn::{FnArg, PatType, ReturnType};
use crate::type_mapping::map_type;
use crate::quote_helper::{is_returning_string, string_function_wrapper_quote, function_wrapper_quote,
                          register_func_meta_quote, process_arg};
use crate::attr_parser::{extract_needs_context, extract_params};

#[proc_macro_attribute]
pub fn udf(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut needs_context = false;
    let _ = extract_needs_context(attrs, &mut needs_context);
    let input = proc_macro2::TokenStream::from(input);
    udf_impl(input, needs_context).into()
}

fn udf_impl(input: proc_macro2::TokenStream, mut needs_context: bool) -> proc_macro2::TokenStream {
    let function = extract_params(input);
    let function_name = &function.sig.ident;
    let return_type = &function.sig.output;

    let mut wrapper_args = Vec::new();
    let mut call_args = Vec::new();
    let mut arg_types = Vec::new();
    let is_returning_string = is_returning_string(return_type);
    needs_context = needs_context || is_returning_string;

    if needs_context {
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
                string_function_wrapper_quote(&function, &wrapper_name, &mut wrapper_args, &function_name, &mut call_args)
            } else {
                function_wrapper_quote(&function, &wrapper_name, &mut wrapper_args, &function_name, &mut call_args, ty)
            };
            let register_func_meta = register_func_meta_quote(&function_name, &arg_types,
                                                              &wrapper_name, needs_context, &return_arrow_type);
            quote! {
                #wrapper_func
                #register_func_meta
            }
        }
    };

    expanded
}
