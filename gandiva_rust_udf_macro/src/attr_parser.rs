use syn::{ItemFn, parse_macro_input};

// needs_context can now be automatically determined by return_type, and we may not need this function anymore
pub(crate) fn extract_needs_context(
    attrs: proc_macro::TokenStream,
    needs_context: &mut bool,
) -> proc_macro::TokenStream {
    let udf_attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("needs_context") {
            *needs_context = true;
            Ok(())
        } else {
            Err(meta.error("unsupported udf macro attribute"))
        }
    });

    parse_macro_input!(attrs with udf_attr_parser);
    proc_macro::TokenStream::new()
}

pub(crate) fn extract_params(input: proc_macro2::TokenStream) -> ItemFn {
    syn::parse2(input).unwrap()
}