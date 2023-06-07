#[cfg(test)]
mod tests {
    use crate::udf_impl;
    use crate::context_fns_impl;

    #[test]
    fn test_no_arg_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf() -> f64 {
                1.0
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_() -> f64 {
                my_udf()
            }
        };
        let actual = udf_impl(input, false);
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
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_int64(x: i64) -> f64 {
                my_udf(x)
            }
        };
        let actual = udf_impl(input, false);
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
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_int64_int32(x: i64, y: i32) -> f32 {
                my_udf(x, y)
            }
        };
        let actual = udf_impl(input, false);
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
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_utf8(x: *const libc::c_char, x_len: i32) -> bool {
                my_udf(std::str::from_utf8(
                    unsafe { std::slice::from_raw_parts(x as *const u8, x_len as usize) }
                ).unwrap())
            }
        };
        let actual = udf_impl(input, false);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_needs_context_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: &str) -> bool {
                true
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_utf8(ctx: i64, x: *const libc::c_char, x_len: i32) -> bool {
                my_udf(std::str::from_utf8(
                    unsafe { std::slice::from_raw_parts(x as *const u8, x_len as usize) }
                ).unwrap())
            }
        };
        let actual = udf_impl(input, true);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_needs_context_return_string_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: i64) -> String {
                String::from("hello")
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_int64(ctx: i64, x: i64, out_len: *mut i32) -> *mut libc::c_char {
                let result = my_udf(x);
                return_gdv_string(ctx, &result, out_len)
            }
        };
        let actual = udf_impl(input, true);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_gandiva_context_fns() {
        let context_fns = context_fns_impl();
        assert!(!context_fns.is_empty());
    }
}