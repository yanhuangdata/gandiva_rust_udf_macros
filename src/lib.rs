mod udf_macro_test;

extern crate proc_macro;

use quote::{quote, format_ident};
use syn::ItemFn;

#[proc_macro]
pub fn udf(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    udf_impl(input).into()
}

pub fn udf_impl(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let function: ItemFn = syn::parse2(input).unwrap();
    let function_name = &function.sig.ident;
    let return_type = &function.sig.output;
    let wrapper_name = format_ident!("{}_void", function_name);

    let expanded = quote! {
        #[no_mangle]
        pub extern "C" fn #wrapper_name() #return_type {
            #function_name()
        }
    };
    expanded
}
