#[cfg(test)]
mod tests {
    use gandiva_rust_udf_shared::udf_registry_generator::generate_udf_registry;
    use std::path::PathBuf;

    fn scan_dir(dir: &str) -> PathBuf {
        let mut data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        data_path.push("tests");
        data_path.push("data");
        if !dir.is_empty() {
            data_path.push(dir);
        }
        data_path
    }

    #[test]
    fn test_generate_no_parameter_udf_registry() {
        let expected: proc_macro2::TokenStream = quote::quote! {
            #[udf_registry]
            pub fn register_all_funcs() {
                my_foo_func::register_foo_func_();
            }
        };
        let actual = generate_udf_registry(&scan_dir("foo_func"));
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_single_parameter_udf_registry() {
        let expected: proc_macro2::TokenStream = quote::quote! {
            #[udf_registry]
            pub fn register_all_funcs() {
                my_bar_func::register_bar_func_int32();
            }
        };
        let actual = generate_udf_registry(&scan_dir("bar_func"));
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_generate_multi_udfs_registry() {
        let expected: proc_macro2::TokenStream = quote::quote! {
            #[udf_registry]
            pub fn register_all_funcs() {
                my_bar_func::register_bar_func_int32();
                my_foo_func::register_foo_func_();
            }
        };
        // use empty string to scan the whole directory
        let actual = generate_udf_registry(&scan_dir(""));
        assert_eq!(actual.to_string(), expected.to_string());
    }
}
