use gandiva_rust_udf_macro::udf;

#[udf]
pub fn bar_func(x: i32) -> i64 {
    42
}