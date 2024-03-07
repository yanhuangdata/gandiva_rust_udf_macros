// map type from Rust name into Arrow type name
pub fn map_type(arg_type: &str) -> String {
    match arg_type {
        "& str" => "utf8",
        "bool" => "boolean",
        "i8" => "int8",
        "i16" => "int16",
        "i32" => "int32",
        "i64" => "int64",
        "u8" => "uint8",
        "u16" => "uint16",
        "u32" => "uint32",
        "u64" => "uint64",
        "f32" => "float32",
        "f64" => "float64",
        _ => arg_type,
    }.to_string()
}
