# Gandiva Rust UDF Macros
This crate provides a `udf` macro to simplify the creation of Gandiva UDFs in Rust.

# Example
```rust
use gandiva_rust_udf_macro::udf;
use std::net::Ipv4Addr;

#[udf]
fn is_ipv4(addr: &str) -> bool {
    addr.parse::<Ipv4Addr>().is_ok()
}
```

The macro takes advantage of the Gandiva external C function capability [1], and the Rust function defined with the `udf` macro can be used as a Gandiva UDF.

# Supported data types in UDF
## Input parameters types
- `bool`
- `i8`
- `i16`
- `i32`
- `i64`
- `f32`
- `f64`
- `&str`
## Return value types
- `bool`
- `i8`
- `i16`
- `i32`
- `i64`
- `f32`
- `f64`
- `String`
- `Result`
  - If your function returns a `Result`, the error will be propagated to the caller.
  - The `Result` generics will have two types, the first one is the success value type, and the second one is the error type.
    - For example, `Result<i64, String>` means the function returns a `Result` with `i64` as the success value type and `String` as the error type.
# References
[1] Gandiva External Function Development Guide, https://arrow.apache.org/docs/cpp/gandiva/external_func.html