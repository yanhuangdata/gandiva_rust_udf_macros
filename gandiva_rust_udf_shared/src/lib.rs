use std::ffi::CString;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

#[allow(dead_code)]
// implement serialized and deserialized for UdfMetaData
#[derive(Serialize, Deserialize, Debug)]
pub struct UdfMetaData {
    pub base_name: String,
    pub param_types: Vec<String>,
    pub return_type: String,
    pub pc_name: String,
}

pub static mut GDV_FN_CONTEXT_ARENA_MALLOC: Option<unsafe extern "C" fn(i64, i32) -> *mut i8> = None;
pub static mut GDV_FN_CONTEXT_SET_ERROR_MSG: Option<unsafe extern "C" fn(i64, *const i8)> = None;

#[no_mangle]
pub extern "C" fn initialize_gdv_fn_context(malloc_ptr: unsafe extern "C" fn(i64, i32) -> *mut i8,
                                            set_error_msg_ptr: unsafe extern "C" fn(i64, *const i8)) {
    unsafe {
        GDV_FN_CONTEXT_ARENA_MALLOC = Some(malloc_ptr);
        GDV_FN_CONTEXT_SET_ERROR_MSG = Some(set_error_msg_ptr);
    }
}

#[allow(dead_code)]
fn return_gdv_string(ctx: i64, result: &str, out_len: *mut i32) -> *mut libc::c_char {
    let result_len = result.len() as i32;
    let result_ptr = unsafe {
        if let Some(context_arena_malloc) = GDV_FN_CONTEXT_ARENA_MALLOC {
            context_arena_malloc(ctx, result_len)
        } else {
            eprintln!("GDV_FN_CONTEXT_ARENA_MALLOC is not set");
            *out_len = 0;
            return std::ptr::null_mut();
        }
    };
    if result_ptr.is_null() {
        unsafe {
            if let Some(context_set_error_msg) = GDV_FN_CONTEXT_SET_ERROR_MSG {
                context_set_error_msg(ctx, "Memory allocation failed".as_ptr() as *const libc::c_char);
            } else {
                eprintln!("GDV_FN_CONTEXT_SET_ERROR_MSG is not set");
            }
            *out_len = 0;
        }
        return std::ptr::null_mut();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(result.as_ptr() as *const u8, result_ptr as *mut u8, result_len as usize);
        *out_len = result_len;
    }
    result_ptr
}

lazy_static! {
    pub static ref UDF_REGISTRY: std::sync::Mutex<Vec<UdfMetaData>> = std::sync::Mutex::new(vec![]);
}

pub fn register_udf(udf_meta_data: UdfMetaData) {
    let mut udf_registry = UDF_REGISTRY.lock().unwrap();
    udf_registry.push(udf_meta_data);
}

// return serialized json string of UdfMetaData list, no need to use gandiva's malloc
// this generates a function to retrieve the udf metadata as a json string
// caller of this API should free the memory
pub extern "C" fn get_udf_registry() -> *mut libc::c_char {
    let udf_registry = UDF_REGISTRY.lock().unwrap();
    let json_str = serde_json::to_string(&*udf_registry).unwrap();

    let c_str = CString::new(json_str).unwrap();
    let ptr = c_str.into_raw();
    ptr
}