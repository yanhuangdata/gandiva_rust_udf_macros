#[cfg(test)]
mod tests {
    use gandiva_rust_udf_build::generate_udf_registry_and_dependencies;
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
        let expected_regs: proc_macro2::TokenStream = quote::quote! {
            #[udf_registry]
            pub fn register_all_funcs() {
                my_foo_func::register_foo_func_();
            }
        };
        let expected_deps = r#"
my_foo_func = { path = "../foo_func" }"#;
        let (actual_regs, actual_deps) = generate_udf_registry_and_dependencies(&scan_dir("foo_func"));
        assert_eq!(actual_regs.to_string(), expected_regs.to_string());
        assert_eq!(actual_deps, expected_deps);
    }

    #[test]
    fn test_generate_single_parameter_udf_registry() {
        let expected_regs: proc_macro2::TokenStream = quote::quote! {
            #[udf_registry]
            pub fn register_all_funcs() {
                my_bar_func::register_bar_func_int32();
            }
        };
        let expected_deps = r#"
my_bar_func = { path = "../bar_func" }"#;
        let (actual_regs, actual_deps) = generate_udf_registry_and_dependencies(&scan_dir("bar_func"));
        assert_eq!(actual_regs.to_string(), expected_regs.to_string());
        assert_eq!(actual_deps, expected_deps);
    }

    #[test]
    fn test_generate_multi_udfs_registry() {
        let expected_regs: proc_macro2::TokenStream = quote::quote! {
            #[udf_registry]
            pub fn register_all_funcs() {
                my_bar_func::register_bar_func_int32();
                my_foo_func::register_foo_func_();
            }
        };
        let expected_deps = r#"
my_bar_func = { path = "../bar_func" }
my_foo_func = { path = "../foo_func" }"#;
        // use empty string to scan the whole directory
        let (actual_regs, actual_deps) = generate_udf_registry_and_dependencies(&scan_dir(""));
        assert_eq!(actual_regs.to_string(), expected_regs.to_string());
        assert_eq!(actual_deps, expected_deps);
    }
}
