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

# References
[1] Gandiva External Function Development Guide, https://arrow.apache.org/docs/cpp/gandiva/external_func.html