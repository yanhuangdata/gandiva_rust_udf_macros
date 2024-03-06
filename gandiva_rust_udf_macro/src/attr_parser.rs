use syn::meta::ParseNestedMeta;
use syn::{parse_macro_input, ItemFn, Lit, LitBool, LitStr, Meta};

fn _parse_udf_meta(
    attrs: proc_macro::TokenStream,
    name: &mut Option<String>,
    aliases: &mut Vec<String>,
    needs_context: &mut bool,
) -> proc_macro::TokenStream {
    let udf_attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            let value = meta.value()?;
            let s: LitStr = value.parse()?;
            *name = Some(s.value());
            Ok(())
        } else if meta.path.is_ident("aliases") {
            let value = meta.value()?;
            // let nested_meta = ParseNestedMeta::parse(&value)?;
            Ok(())
        } else if meta.path.is_ident("needs_context") {
            let value = meta.value()?;
            let s: LitBool = value.parse()?;
            *needs_context = s.value;
            Ok(())
        } else {
            // return an error if the attribute is not supported with meta's path
            Err(meta.error("unsupported udf macro attribute"))
        }
    });
    parse_macro_input!(attrs with udf_attr_parser);
    proc_macro::TokenStream::new()
}

// Extract UDF meta from the #[udf(name="my_func", aliases = ("my_func1", "my_func2"))] macro attributes, including:
// 1) name
// 2) aliases
// 3) needs_context, needs_context can now be automatically determined by return_type
// return a tuple of (name, aliases, needs_context)
pub(crate) fn extract_udf_meta(
    attrs: proc_macro::TokenStream,
) -> (Option<String>, Vec<String>, bool) {
    let mut name = None;
    let mut aliases = Vec::new();
    let mut needs_context = false;
    _parse_udf_meta(attrs, &mut name, &mut aliases, &mut needs_context);
    (name, aliases, needs_context)
}

pub(crate) fn extract_params(input: proc_macro2::TokenStream) -> ItemFn {
    syn::parse2(input).unwrap()
}
