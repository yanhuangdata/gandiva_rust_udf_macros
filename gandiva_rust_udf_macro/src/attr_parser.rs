use syn::parse::{Parser};
use syn::{Attribute, ItemFn, LitBool, LitStr};

// Extract UDF meta from the #[udf(name="my_func", aliases = ["my_func1", "my_func2"])] macro attributes, including:
// 1) name
// 2) aliases
// 3) needs_context, needs_context can now be automatically determined by return_type
// return a tuple of (name, aliases, needs_context)
pub(crate) fn extract_udf_meta(
    input: proc_macro2::TokenStream,
) -> Result<(Option<String>, Vec<String>, bool, Option<String>), syn::Error> {
    let mut name = None;
    let mut aliases = Vec::new();
    let mut needs_context = false;
    let mut result_nullable = None;

    // this is a workaround to parse the attributes
    // https://github.com/dtolnay/syn/issues/359
    let attr_text = format!("#[udf({})]", input.to_string());
    let attrs = Attribute::parse_outer.parse2(attr_text.parse()?)?;
    for attr in attrs {
        if attr.path().is_ident("udf") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    name = Some(s.value());
                    Ok(())
                } else if meta.path.is_ident("aliases") {
                    let value = meta.value()?;
                    let list: syn::ExprArray = value.parse()?;
                    for expr in list.elems {
                        if let syn::Expr::Lit(expr_lit) = expr {
                            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                aliases.push(lit_str.value());
                            } else {
                                return Err(syn::Error::new_spanned(
                                    expr_lit,
                                    "Expected string literal for function alias",
                                ));
                            }
                        } else {
                            return Err(syn::Error::new_spanned(
                                expr,
                                "Expected string literal for function alias",
                            ));
                        }
                    }
                    Ok(())
                } else if meta.path.is_ident("needs_context") {
                    let value = meta.value()?;
                    let b: LitBool = value.parse()?;
                    needs_context = b.value;
                    Ok(())
                } else if meta.path.is_ident("result_nullable") {
                    let value = meta.value()?;
                    let s: LitStr = value.parse()?;
                    result_nullable = Some(s.value());
                    // only if_null/never/internal are allowed
                    if result_nullable.as_ref().unwrap() != "if_null"
                        && result_nullable.as_ref().unwrap() != "never"
                        && result_nullable.as_ref().unwrap() != "internal"
                    {
                        return Err(syn::Error::new_spanned(
                            meta.path,
                            "Unsupported value for result_nullable attribute. \
                            Only if_null, never, internal are allowed.",
                        ));
                    }
                    Ok(())
                } else {
                    Err(syn::Error::new_spanned(
                        meta.path,
                        "Unknown attribute for UDF function",
                    ))
                }
            })?;
        }
    }
    Ok((name, aliases, needs_context, result_nullable))
}

pub(crate) fn extract_params(input: proc_macro2::TokenStream) -> ItemFn {
    syn::parse2(input).unwrap()
}
