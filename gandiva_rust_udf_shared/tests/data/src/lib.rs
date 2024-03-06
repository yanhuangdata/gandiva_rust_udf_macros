use gandiva_rust_udf_macro::udf;

#[udf]
pub fn root_func() -> i64 {
    42
}