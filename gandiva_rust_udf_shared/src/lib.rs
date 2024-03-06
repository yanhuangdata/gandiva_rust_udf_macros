use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::ffi::CString;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DataType {
    pub type_name: String,
    // optional, for `list` only
    pub value_type: Option<Box<DataType>>,
    // optional, for `decimal` only
    pub precision: Option<i32>,
    // optional, for `decimal` only
    pub scale: Option<i32>,
    // optional, for `timestamp` only
    pub unit: Option<String>,
}

impl Default for crate::DataType {
    fn default() -> Self {
        DataType {
            type_name: Default::default(),
            value_type: None,
            precision: None,
            scale: None,
            unit: None,
        }
    }
}

#[allow(dead_code)]
// implement serialized and deserialized for UdfMetaData
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UdfMetaData {
    pub name: String,
    pub aliases: Vec<String>,
    pub param_types: Vec<DataType>,
    pub return_type: DataType,
    pub pc_name: String,
    pub result_nullable: String,
    pub can_return_errors: bool,
    pub needs_context: bool,
}

impl Default for UdfMetaData {
    fn default() -> Self {
        UdfMetaData {
            name: Default::default(),
            aliases: vec![],
            param_types: vec![],
            return_type: DataType::default(),
            pc_name: Default::default(),
            result_nullable: "if_null".to_string(),
            can_return_errors: false,
            needs_context: false,
        }
    }
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct UdfRegistry {
    pub version: String,
    pub functions: Vec<UdfMetaData>,
}

pub static mut GDV_FN_CONTEXT_ARENA_MALLOC: Option<unsafe extern "C" fn(i64, i32) -> *mut i8> =
    None;
pub static mut GDV_FN_CONTEXT_SET_ERROR_MSG: Option<unsafe extern "C" fn(i64, *const i8)> = None;

#[no_mangle]
pub extern "C" fn initialize_gdv_fn_context(
    malloc_ptr: unsafe extern "C" fn(i64, i32) -> *mut i8,
    set_error_msg_ptr: unsafe extern "C" fn(i64, *const i8),
) {
    unsafe {
        GDV_FN_CONTEXT_ARENA_MALLOC = Some(malloc_ptr);
        GDV_FN_CONTEXT_SET_ERROR_MSG = Some(set_error_msg_ptr);
    }
}

// this function is used by the `udf` macro, if the Rust UDF returns a String,
// it will be converted into a C string (returning an i32 output parameter as string length, and libc::c_char array as data) using this function
#[allow(dead_code)]
pub fn return_gdv_string(ctx: i64, result: &str, out_len: *mut i32) -> *mut libc::c_char {
    let result_len = result.len() as i32;
    let result_ptr = unsafe {
        if let Some(context_arena_malloc) = GDV_FN_CONTEXT_ARENA_MALLOC {
            context_arena_malloc(ctx, result_len)
        } else {
            eprintln!("GDV_FN_CONTEXT_ARENA_MALLOC is not set");
            *out_len = 0;
            return std::ptr::null_mut();
        }
    } as *mut libc::c_char;
    if result_ptr.is_null() {
        unsafe {
            if let Some(context_set_error_msg) = GDV_FN_CONTEXT_SET_ERROR_MSG {
                context_set_error_msg(ctx, "Memory allocation failed".as_ptr() as *const i8);
            } else {
                eprintln!("GDV_FN_CONTEXT_SET_ERROR_MSG is not set");
            }
            *out_len = 0;
        }
        return std::ptr::null_mut();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            result.as_ptr() as *const u8,
            result_ptr as *mut u8,
            result_len as usize,
        );
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
    let udfs = UDF_REGISTRY.lock().unwrap();
    // get the Vec<UdfMetaData> from the mutex
    let udf_registry = UdfRegistry {
        version: "1.0".to_string(),
        functions: udfs.clone(),
    };
    let registry_json = serde_json::to_string(&udf_registry).unwrap();
    let c_str = CString::new(registry_json).unwrap();
    let ptr = c_str.into_raw();
    ptr
}

pub extern "C" fn free_udf_registry(ptr: *mut libc::c_char) {
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}
