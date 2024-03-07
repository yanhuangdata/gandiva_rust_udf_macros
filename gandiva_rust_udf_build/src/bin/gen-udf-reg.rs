use std::env;
use std::path::Path;

use gandiva_rust_udf_build::generate_udf_registry;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        match env::current_dir() {
            Ok(result) => {
                println!("generating in current dir: {}", result.to_str().unwrap());
                generate_udf_registry(&result);
            }
            Err(e) => println!("failed to get current dir: {:?}", e),
        }
        return;
    }
    let path_str = args.get(1).unwrap();
    println!("generating in given dir: {}", path_str);
    let path = Path::new(path_str);
    generate_udf_registry(&path);
}