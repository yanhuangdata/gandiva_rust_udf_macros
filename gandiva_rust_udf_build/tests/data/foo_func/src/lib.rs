use gandiva_rust_udf_macro::udf;

#[udf]
pub fn foo_func() -> i64 {
    42
}