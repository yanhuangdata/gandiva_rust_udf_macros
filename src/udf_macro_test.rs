#[cfg(test)]
mod tests {
    use crate::udf_impl;

    #[test]
    fn test_no_arg_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_no_arg_udf() -> f64 {
                1.0
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #[no_mangle]
            pub extern "C" fn my_no_arg_udf_void() -> f64 {
                my_no_arg_udf()
            }
        };
        let actual = udf_impl(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }
}