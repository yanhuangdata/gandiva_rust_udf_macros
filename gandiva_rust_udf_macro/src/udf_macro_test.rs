#[cfg(test)]
mod macro_tests {
    use crate::extract_udf_meta;
    use crate::udf_impl;
    use crate::udf_registry_impl;
    use default_args::default_args;

    default_args! {
        fn gen_udf(
            input: proc_macro2::TokenStream,
            name: Option<String> = None,
            aliases: Vec<String> = Vec::new(),
            needs_context: bool = false,
            can_return_errors: bool = false,
            result_nullable: Option<String> = None,
        ) -> proc_macro2::TokenStream {
            udf_impl(input, name, aliases, needs_context, can_return_errors, result_nullable)
        }
    }

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
                let result = my_udf();
                result
            }

            pub fn register_my_udf_() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec![],
                    param_types: vec![],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "float64".to_string(), ..Default::default() },
                    pc_name: "my_udf_".to_string(),
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input);
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
                let result = my_udf(x);
                result
            }

            pub fn register_my_udf_int64() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec![],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "int64".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "float64".to_string(), ..Default::default() },
                    pc_name: "my_udf_int64".to_string(),
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input);
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
                let result = my_udf(x);
                result
            }

            pub fn register_my_udf_boolean() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec![],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "boolean".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "boolean".to_string(), ..Default::default() },
                    pc_name: "my_udf_boolean".to_string(),
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input);
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
                let result = my_udf(x, y);
                result
            }

            pub fn register_my_udf_int64_int32() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec![],
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
        let actual = gen_udf!(input);
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
                let result = my_udf(std::str::from_utf8(
                    unsafe { std::slice::from_raw_parts(x as *const u8, x_len as usize) }
                ).unwrap());
                result
            }

            pub fn register_my_udf_utf8() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec![],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "utf8".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "boolean".to_string(), ..Default::default() },
                    pc_name: "my_udf_utf8".to_string(),
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input);
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
                let result = my_udf(std::str::from_utf8(
                    unsafe { std::slice::from_raw_parts(x as *const u8, x_len as usize) }
                ).unwrap());
                result
            }

            pub fn register_my_udf_utf8() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec![],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "utf8".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "boolean".to_string(), ..Default::default() },
                    pc_name: "my_udf_utf8".to_string(),
                    needs_context: true,
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input, needs_context = true);
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
                    aliases: vec![],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "int64".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "utf8".to_string(), ..Default::default() },
                    pc_name: "my_udf_int64".to_string(),
                    needs_context: true,
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input);
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

    #[test]
    fn test_customized_base_name_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: i64) -> f64 {
                1.0
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_int64(x: i64) -> f64 {
                let result = my_udf(x);
                result
            }

            pub fn register_my_udf_int64() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "your_udf".to_string(),
                    aliases: vec![],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "int64".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "float64".to_string(), ..Default::default() },
                    pc_name: "my_udf_int64".to_string(),
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input, name = Some("your_udf".to_string()));
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_alias_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: i64) -> f64 {
                1.0
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_int64(x: i64) -> f64 {
                let result = my_udf(x);
                result
            }

            pub fn register_my_udf_int64() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec!["your_udf".to_string()],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "int64".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "float64".to_string(), ..Default::default() },
                    pc_name: "my_udf_int64".to_string(),
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input, aliases = vec!["your_udf".to_string()]);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_result_nullable_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: i64) -> f64 {
                1.0
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_int64(x: i64) -> f64 {
                let result = my_udf(x);
                result
            }

            pub fn register_my_udf_int64() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec![],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "int64".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "float64".to_string(), ..Default::default() },
                    pc_name: "my_udf_int64".to_string(),
                    result_nullable: "never".to_string(),
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input, result_nullable = Some("never".to_string()));
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_extract_udf_meta() {
        let input: proc_macro2::TokenStream = quote::quote! {
            name = "my_udf",
            aliases = ["your_udf"],
            needs_context = true,
            result_nullable = "never"
        };
        let expected = (
            Some("my_udf".to_string()),
            vec!["your_udf".to_string()],
            true,
            false,
            Some("never".to_string()),
        );
        let actual = extract_udf_meta(input);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_extract_invalid_alias() {
        let input: proc_macro2::TokenStream = quote::quote! {
            aliases = [42]
        };
        let actual = extract_udf_meta(input);
        // assert error occurs
        assert_eq!(
            actual.err().unwrap().to_string(),
            "Expected string literal for function alias"
        );
    }

    #[test]
    fn test_extract_invalid_attribute() {
        let input: proc_macro2::TokenStream = quote::quote! {
            no_such_attr = 42
        };
        let actual = extract_udf_meta(input);
        assert_eq!(
            actual.err().unwrap().to_string(),
            "Unknown attribute for UDF function"
        );
    }

    #[test]
    fn test_extract_name_should_be_string() {
        let input: proc_macro2::TokenStream = quote::quote! {
            name = 42
        };
        let actual = extract_udf_meta(input);
        assert_eq!(actual.err().unwrap().to_string(), "expected string literal");
    }

    #[test]
    fn test_extract_alias_should_be_array() {
        let input: proc_macro2::TokenStream = quote::quote! {
            aliases = "your_udf"
        };
        let actual = extract_udf_meta(input);
        assert_eq!(
            actual.err().unwrap().to_string(),
            "expected square brackets"
        );
    }

    #[test]
    fn test_extract_result_nullable_should_be_string() {
        let input: proc_macro2::TokenStream = quote::quote! {
            result_nullable = "if_null"
        };
        let expected = (None, vec![], false, false, Some("if_null".to_string()));
        let actual = extract_udf_meta(input);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_extract_invalid_result_nullable_value() {
        let input: proc_macro2::TokenStream = quote::quote! {
            result_nullable = "no_supported_nullable_value"
        };
        let actual = extract_udf_meta(input);
        assert_eq!(
            actual.err().unwrap().to_string(),
            "Unsupported value for result_nullable attribute. Only if_null, never, internal are allowed."
        );
    }

    #[test]
    fn test_extract_needs_context_should_be_bool() {
        let input: proc_macro2::TokenStream = quote::quote! {
            needs_context = 42
        };
        let actual = extract_udf_meta(input);
        assert_eq!(
            actual.err().unwrap().to_string(),
            "expected boolean literal"
        );
    }

    #[test]
    fn test_extract_udf_meta_multi_aliases_false_needs_context() {
        let input: proc_macro2::TokenStream = quote::quote! {
            name = "my_udf",
            aliases = ["your_udf", "her_udf"],
            needs_context = false,
            result_nullable = "internal"
        };
        let expected = (
            Some("my_udf".to_string()),
            vec!["your_udf".to_string(), "her_udf".to_string()],
            false,
            false,
            Some("internal".to_string()),
        );
        let actual = extract_udf_meta(input);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_extract_udf_meta_default() {
        let input: proc_macro2::TokenStream = quote::quote! {};
        let expected = (
            None,
            vec![],
            false,
            false,
            None,
        );
        let actual = extract_udf_meta(input);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_extract_udf_meta_can_return_errors() {
        let input: proc_macro2::TokenStream = quote::quote! {
            can_return_errors = true
        };
        let expected = (
            None,
            vec![],
            false,
            true,
            None,
        );
        let actual = extract_udf_meta(input);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn test_return_error_bool_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: &str) -> Result<bool, String> {
                true
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_utf8(ctx: i64, x: *const libc::c_char, x_len: i32) -> bool {
                let result = my_udf(std::str::from_utf8(
                    unsafe { std::slice::from_raw_parts(x as *const u8, x_len as usize) }
                ).unwrap());
                match result {
                    Ok(return_value) => return_value,
                    Err(err) => {
                        gandiva_rust_udf_shared::set_error_msg(ctx, &err);
                        false
                    }
                }
            }

            pub fn register_my_udf_utf8() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec![],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "utf8".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "boolean".to_string(), ..Default::default() },
                    pc_name: "my_udf_utf8".to_string(),
                    needs_context: true,
                    can_return_errors: true,
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_return_error_int32_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: &str) -> Result<i32, String> {
                true
            }
        };

        // int32 return value uses 0.into() as the default return value
        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_utf8(ctx: i64, x: *const libc::c_char, x_len: i32) -> i32 {
                let result = my_udf(std::str::from_utf8(
                    unsafe { std::slice::from_raw_parts(x as *const u8, x_len as usize) }
                ).unwrap());
                match result {
                    Ok(return_value) => return_value,
                    Err(err) => {
                        gandiva_rust_udf_shared::set_error_msg(ctx, &err);
                        0.into()
                    }
                }
            }

            pub fn register_my_udf_utf8() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec![],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "utf8".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "int32".to_string(), ..Default::default() },
                    pc_name: "my_udf_utf8".to_string(),
                    needs_context: true,
                    can_return_errors: true,
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_return_error_string_udf() {
        let input: proc_macro2::TokenStream = quote::quote! {
            pub fn my_udf(x: i64) -> Result<String, String> {
                String::from("hello")
            }
        };

        let expected: proc_macro2::TokenStream = quote::quote! {
            #input

            #[no_mangle]
            pub extern "C" fn my_udf_int64(ctx: i64, x: i64, out_len: *mut i32) -> *mut libc::c_char {
                let result = my_udf(x);
                match result {
                    Ok(value) => gandiva_rust_udf_shared::return_gdv_string(ctx, &value, out_len),
                    Err(err) => {
                        gandiva_rust_udf_shared::set_error_msg(ctx, &err);
                        unsafe {
                            *out_len = 0;
                        }
                        std::ptr::null_mut()
                    }
                }
            }

            pub fn register_my_udf_int64() {
                gandiva_rust_udf_shared::register_udf(gandiva_rust_udf_shared::UdfMetaData {
                    name: "my_udf".to_string(),
                    aliases: vec![],
                    param_types: vec![gandiva_rust_udf_shared::DataType { type_name: "int64".to_string(), ..Default::default() }],
                    return_type: gandiva_rust_udf_shared::DataType { type_name: "utf8".to_string(), ..Default::default() },
                    pc_name: "my_udf_int64".to_string(),
                    needs_context: true,
                    can_return_errors: true,
                    ..Default::default()
                });
            }
        };
        let actual = gen_udf!(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
