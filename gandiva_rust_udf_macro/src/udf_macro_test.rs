#[cfg(test)]
mod tests {
    use crate::udf_impl;
    use crate::udf_registry_impl;

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

            pub fn register_my_udf_() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    param_types: vec![],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "float64".to_string(), ..Default::default() },
                    pc_name: "my_udf_".to_string(),
                    ..Default::default()
                });
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

            pub fn register_my_udf_int64() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "int64".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "float64".to_string(), ..Default::default() },
                    pc_name: "my_udf_int64".to_string(),
                    ..Default::default()
                });
            }
        };
        let actual = udf_impl(input, false);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_bool_arg_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: bool) -> bool {
                true
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_boolean(x: bool) -> bool {
                my_udf(x)
            }

            pub fn register_my_udf_boolean() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "boolean".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "boolean".to_string(), ..Default::default() },
                    pc_name: "my_udf_boolean".to_string(),
                    ..Default::default()
                });
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

            pub fn register_my_udf_int64_int32() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    param_types: vec![
                        gandiva_rust_udf_shared::DataType { type_name: "int64".to_string(), ..Default::default() },
                        gandiva_rust_udf_shared::DataType { type_name: "int32".to_string(), ..Default::default() }
                    ],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "float32".to_string(), ..Default::default() },
                    pc_name: "my_udf_int64_int32".to_string(),
                    ..Default::default()
                });
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

            pub fn register_my_udf_utf8() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "utf8".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "boolean".to_string(), ..Default::default() },
                    pc_name: "my_udf_utf8".to_string(),
                    ..Default::default()
                });
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

            pub fn register_my_udf_utf8() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "utf8".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "boolean".to_string(), ..Default::default() },
                    pc_name: "my_udf_utf8".to_string(),
                    needs_context: true,
                    ..Default::default()
                });
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
                gandiva_rust_udf_shared::return_gdv_string(ctx, &result, out_len)
            }

            pub fn register_my_udf_int64() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "int64".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "utf8".to_string(), ..Default::default() },
                    pc_name: "my_udf_int64".to_string(),
                    needs_context: true,
                    ..Default::default()
                });
            }
        };
        let actual = udf_impl(input, true);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_udf_registry_macro() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn register_all_udfs() {
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn load_registered_udfs() -> *mut libc::c_char {
                register_all_udfs();
                let registry_c_str = gandiva_rust_udf_shared::get_udf_registry();
                registry_c_str
            }

            #[no_mangle]
            pub extern "C" fn finish_loading_registered_udfs(registry: *mut libc::c_char) {
                gandiva_rust_udf_shared::free_udf_registry(registry);
            }
        };
        let actual = udf_registry_impl(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }
}