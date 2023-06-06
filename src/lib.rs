mod udf_macro_test;

extern crate proc_macro;

use quote::{quote, format_ident};
use syn::{ItemFn, FnArg, PatType, ReturnType};

#[proc_macro]
pub fn udf(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    udf_impl(input).into()
}

pub fn udf_impl(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let function: ItemFn = syn::parse2(input).unwrap();
    let function_name = &function.sig.ident;
    let return_type = &function.sig.output;
    let mut wrapper_args = Vec::new();
    let mut call_args = Vec::new();

    let mut arg_types = Vec::new();
    for input in &function.sig.inputs {
        if let FnArg::Typed(PatType { ty, pat, .. }) = input {
            let arg_name = pat;
            if quote!(#ty).to_string() == "& str" {
                arg_types.push("utf8".to_string());
                let arg_name_len = format_ident!("{}_len", quote!(#arg_name).to_string());
                wrapper_args.push(quote! { #arg_name: *const c_char, #arg_name_len: i32 });
                call_args.push(quote! { str::from_utf8(slice::from_raw_parts(#arg_name as *const u8, #arg_name_len as usize)).unwrap() });
            } else {
                arg_types.push(quote!(#ty).to_string());
                wrapper_args.push(quote! { #arg_name: #ty });
                call_args.push(quote! { #arg_name });
            }
        }
    }


    let wrapper_name = format_ident!("{}_{}", function_name, arg_types.join("_"));

    let expanded = match return_type {
        ReturnType::Default => {
            quote! {
                #[no_mangle]
                pub extern "C" fn #wrapper_name(#(#wrapper_args),*) {
                    #function_name(#(#call_args),*)
                }
            }
        }
        ReturnType::Type(_, ty) => {
            quote! {
                #[no_mangle]
                pub extern "C" fn #wrapper_name(#(#wrapper_args),*) -> #ty {
                    #function_name(#(#call_args),*)
                }
            }
        }
    };

    expanded
}
