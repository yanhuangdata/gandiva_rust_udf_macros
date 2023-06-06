#[cfg(test)]
mod tests {
    use crate::udf_impl;

    #[test]
    fn test_no_arg_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf() -> f64 {
                1.0
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #[no_mangle]
            pub extern "C" fn my_udf_() -> f64 {
                my_udf()
            }
        };
        let actual = udf_impl(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_i64_arg_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: i64) -> f64 {
                1.0
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #[no_mangle]
            pub extern "C" fn my_udf_i64(x: i64) -> f64 {
                my_udf(x)
            }
        };
        let actual = udf_impl(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_i64_i32_args_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: i64, y: i32) -> f32 {
                1.0
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #[no_mangle]
            pub extern "C" fn my_udf_i64_i32(x: i64, y: i32) -> f32 {
                my_udf(x, y)
            }
        };
        let actual = udf_impl(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_str_arg_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: &str) -> bool {
                true
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #[no_mangle]
            pub extern "C" fn my_udf_utf8(x: *const c_char, x_len: i32) -> bool {
                my_udf(str::from_utf8(slice::from_raw_parts(x as *const u8, x_len as usize)).unwrap())
            }
        };
        let actual = udf_impl(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }
}