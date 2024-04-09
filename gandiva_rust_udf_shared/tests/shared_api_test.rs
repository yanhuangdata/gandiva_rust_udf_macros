#[cfg(test)]
mod tests {
    use gandiva_rust_udf_shared::{
        free_udf_registry, get_udf_registry, initialize_gdv_fn_context, register_udf,
        return_gdv_string, DataType, UdfMetaData, UdfRegistry, UDF_REGISTRY,
    };

    // function used for unit testing purpose
    extern "C" fn gdv_fn_context_arena_malloc(_context: i64, size: i32) -> *mut i8 {
        let mut buffer = Vec::with_capacity(size as usize);
        let ptr = buffer.as_mut_ptr();
        std::mem::forget(buffer);
        ptr as *mut i8
    }

    // function used for unit testing purpose
    extern "C" fn gdv_fn_context_set_error_msg(_context: i64, error_msg: *const i8) {
        unsafe {
            let _ = std::ffi::CStr::from_ptr(error_msg).to_str().unwrap();
        }
    }

    fn _get_udf_meta() -> UdfMetaData {
        UdfMetaData {
            name: "my_udf".to_string(),
            param_types: vec![DataType {
                type_name: "utf8".to_string(),
                ..Default::default()
            }],
            return_type: DataType {
                type_name: "boolean".to_string(),
                ..Default::default()
            },
            pc_name: "my_udf_utf8".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_register_udf() {
        let udf_meta_data = _get_udf_meta();
        register_udf(udf_meta_data);
        let udfs = UDF_REGISTRY.lock().unwrap();
        assert!(udfs.len() >= 1);
        assert_eq!(udfs[0].name, "my_udf");
    }

    #[test]
    fn test_get_udf_registry() {
        let udf_meta_data = _get_udf_meta();
        register_udf(udf_meta_data);
        let registry_c_str = get_udf_registry();
        unsafe {
            let registry = std::ffi::CString::from_raw(registry_c_str);
            let registry_str = registry.to_str().unwrap();
            let udf_registry: UdfRegistry = serde_json::from_str(registry_str).unwrap();
            assert!(udf_registry.functions.len() >= 1);
            assert_eq!(udf_registry.functions[0].name, "my_udf");
        }
    }

    #[test]
    fn test_free_udf_registry() {
        let udf_meta_data = _get_udf_meta();
        register_udf(udf_meta_data);
        let registry_c_str = get_udf_registry();
        free_udf_registry(registry_c_str);
    }

    #[test]
    fn test_initialize_gdv_context_and_return_gdv_string() {
        unsafe {
            initialize_gdv_fn_context(gdv_fn_context_arena_malloc, gdv_fn_context_set_error_msg);
            // out_length variable is used for storing the length of the returned string
            let mut out_length = 0;
            let result = return_gdv_string(0, "hello", &mut out_length);
            let result_c_str = std::ffi::CString::from_raw(result);
            let result_str = result_c_str.to_str().unwrap();
            assert_eq!(result_str, "hello");
            assert_eq!(out_length, 5);
        }

        // result is empty
        unsafe {
            initialize_gdv_fn_context(gdv_fn_context_arena_malloc, gdv_fn_context_set_error_msg);
            // out_length variable is used for storing the length of the returned string
            let mut out_length = 0;
            let result = return_gdv_string(0, "", &mut out_length);
            let result_c_str = std::ffi::CString::from_raw(result);
            let result_str = result_c_str.to_str().unwrap();
            assert_eq!(result_str, "");
            assert_eq!(out_length, 0);
        }
    }
}
