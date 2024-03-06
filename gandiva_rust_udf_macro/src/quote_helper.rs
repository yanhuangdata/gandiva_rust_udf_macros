use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{PatType, ReturnType, Type};
use gandiva_rust_udf_type::map_type;

fn _data_type_quote(type_name: &str) -> proc_macro2::TokenStream {
    quote! {
        gandiva_rust_udf_shared::DataType {
            type_name: #type_name.to_string(),
            ..Default::default()
        }
    }
}

fn _needs_context_quote(needs_context: bool) -> proc_macro2::TokenStream {
    if needs_context {
        quote! { needs_context: true, }
    } else {
        quote! { }
    }
}

pub(crate) fn string_function_wrapper_quote(function: &syn::ItemFn,
                                            wrapper_name: &Ident,
                                            wrapper_args: &mut Vec<proc_macro2::TokenStream>,
                                            function_name: &Ident,
                                            call_args: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    quote! {
        // output the original function
        #function

        #[no_mangle]
        pub extern "C" fn #wrapper_name(#(#wrapper_args),*) -> *mut libc::c_char {
            let result = #function_name(#(#call_args),*);
            gandiva_rust_udf_shared::return_gdv_string(ctx, &result, out_len)
        }
    }
}

pub(crate) fn function_wrapper_quote(function: &syn::ItemFn,
                                     wrapper_name: &Ident,
                                     wrapper_args: &mut Vec<proc_macro2::TokenStream>,
                                     function_name: &Ident,
                                     call_args: &mut Vec<proc_macro2::TokenStream>,
                                     ty: &Box<Type>) -> proc_macro2::TokenStream {
    quote! {
      // output the original function
      #function

      #[no_mangle]
      pub extern "C" fn #wrapper_name(#(#wrapper_args),*) -> #ty {
        #function_name(#(#call_args),*)
      }
    }
}

pub(crate) fn register_func_meta_quote(function_name: &Ident, arg_types: &Vec<String>,
                                       wrapper_name: &Ident, needs_context: bool,
                                       return_arrow_type: &str) -> proc_macro2::TokenStream {
    let base_name_str = function_name.to_string();
    let arg_types_quotes = arg_types.iter().map(|arg_type| { _data_type_quote(arg_type) });
    let pc_name_str = wrapper_name.to_string();
    // register the wrapper function metadata
    let register_func_ident = format_ident!("register_{}", wrapper_name);
    let return_type_quote = _data_type_quote(return_arrow_type);
    // conditionally add needs_context
    let needs_context_quote = _needs_context_quote(needs_context);
    let register_func_meta = quote! {
                pub fn #register_func_ident() {
                    gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                        name: #base_name_str.to_string(),
                        param_types: vec![#(#arg_types_quotes),*],
                        return_type: #return_type_quote,
                        pc_name: #pc_name_str.to_string(),
                        #needs_context_quote
                        ..Default::default()
                    });
                }
            };
    register_func_meta
}

pub(crate) fn is_returning_string(return_type: &ReturnType) -> bool {
    match return_type {
        ReturnType::Default => {
            panic!("The function to be wrapped must have a return type.");
        }
        ReturnType::Type(_, ty) => {
            // if return type is String, use gandiva context function to allocate memory
            let return_type_str = quote!(#ty).to_string();
            if return_type_str == "String" {
                return true;
            }
        }
    };
    false
}

pub(crate) fn process_arg(PatType { ty, pat, .. }: &PatType,
                          wrapper_args: &mut Vec<proc_macro2::TokenStream>,
                          call_args: &mut Vec<proc_macro2::TokenStream>,
                          arg_types: &mut Vec<String>) {
    let arg_name = pat;
    let arg_type = quote!(#ty).to_string();
    // if arg_type is ["i8" | "i16" | "i32" | "i64"] ==> ["int_8" | "int_16" | "int_32" | "int_64"]
    let arg_type_str = arg_type.as_str();
    let mapped_gdv_arg_type = map_type(arg_type_str);

    if mapped_gdv_arg_type == "utf8" {
        let arg_name_len = format_ident!("{}_len", quote!(#arg_name).to_string());
        wrapper_args.push(quote! { #arg_name: *const libc::c_char, #arg_name_len: i32 });
        call_args.push(quote! { std::str::from_utf8(
                    unsafe { std::slice::from_raw_parts(#arg_name as *const u8, #arg_name_len as usize) }
                ).unwrap() });
    } else {
        wrapper_args.push(quote! { #arg_name: #ty });
        call_args.push(quote! { #arg_name });
    }
    arg_types.push(mapped_gdv_arg_type);
}

pub(crate) fn load_registered_udfs_quote(function: syn::ItemFn) -> proc_macro2::TokenStream {
    let registry_function_name = &function.sig.ident;

    quote! {
        #function

        #[no_mangle]
        pub extern "C" fn load_registered_udfs() -> *mut libc::c_char {
            #registry_function_name();
            let registry_c_str = gandiva_rust_udf_shared::get_udf_registry();
            registry_c_str
        }

        #[no_mangle]
        pub extern "C" fn finish_loading_registered_udfs(registry: *mut libc::c_char) {
            gandiva_rust_udf_shared::free_udf_registry(registry);
        }
    }
}