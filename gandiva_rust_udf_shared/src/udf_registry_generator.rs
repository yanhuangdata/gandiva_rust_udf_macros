use std::collections::HashMap;
use crate::type_mapping::map_type;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::fs;
use std::path::Path;
use syn::{visit::Visit, File, FnArg, ItemFn};
use toml::Value;
use walkdir::WalkDir;
use strfmt::strfmt;

struct UdfCollector {
    udf_registration_funcs: Vec<String>,
}

impl<'ast> Visit<'ast> for UdfCollector {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        for attr in &i.attrs {
            if attr.path().is_ident("udf") {
                let fn_name = i.sig.ident.to_string();
                let mut arg_types = Vec::new();
                for input in &i.sig.inputs {
                    match input {
                        FnArg::Typed(pat_type) => {
                            let ty = &pat_type.ty;
                            let arg_type = quote!(#ty).to_string();
                            let arg_type_str = arg_type.as_str();
                            let mapped_gdv_arg_type = map_type(arg_type_str);
                            arg_types.push(mapped_gdv_arg_type);
                        }
                        _ => {
                            panic!("Unsupported function argument type");
                        }
                    }
                }
                let wrapper_name = format_ident!("{}_{}", fn_name, arg_types.join("_"));
                self.udf_registration_funcs.push(wrapper_name.to_string());
                break;
            }
        }
        syn::visit::visit_item_fn(self, i);
    }
}

fn _get_package_dir_name(path: &Path) -> Option<String> {
    if let Some(dir_name) = path.file_name() {
        Some(dir_name.to_string_lossy().into_owned())
    } else {
        None
    }
}

// return (package_name, package_dir)
fn _get_cargo_package_name_and_dir(path: &Path) -> Option<(String, String)> {
    let cargo_toml_path = path.join("Cargo.toml");
    if path.is_dir() && cargo_toml_path.exists() {
        // found a Cargo.toml, which may be a workspace for UDF
        let cargo_toml_contents =
            fs::read_to_string(cargo_toml_path).expect("Failed to read Cargo.toml");
        let cargo_toml: Value = cargo_toml_contents
            .parse()
            .expect("Failed to parse Cargo.toml");
        if let Some(package) = cargo_toml.get("package").and_then(|pkg| pkg.get("name")) {
            if let Some(name) = package.as_str() {
                if let Some(dir_name) = _get_package_dir_name(path) {
                    return Some((name.to_string(), dir_name));
                }
            }
        }
    }
    None
}

// use an ordered map <package_name, Vec<String>> to store all packages with their UDFs within
// the map is ordered so that the generated code is deterministic
fn _extract_package_and_udfs(root_dir: &Path) -> std::collections::BTreeMap<String, (Vec<String>, String)> {
    let mut package_udfs = std::collections::BTreeMap::new();
    for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if let Some((package_name, package_dir)) = _get_cargo_package_name_and_dir(path) {
            let mut collector = UdfCollector {
                udf_registration_funcs: Vec::new(),
            };
            for sub_entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let sub_path = sub_entry.path();
                match _get_cargo_package_name_and_dir(sub_path) {
                    Some((sub_package_name, _)) if sub_package_name != package_name => break,
                    _ => {}
                }
                if sub_path.is_file()
                    && sub_path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs")
                {
                    let content = fs::read_to_string(sub_path).expect("Failed to read file");
                    let syntax_tree: File = syn::parse_str(&content).expect("Failed to parse file");
                    syn::visit::visit_file(&mut collector, &syntax_tree);
                }
            }
            if collector.udf_registration_funcs.len() <= 0 {
                continue;
            }
            package_udfs.insert(package_name, (collector.udf_registration_funcs, package_dir));
        }
    }
    package_udfs
}

fn _collect_udf_registration_funcs(
    package_udfs: &std::collections::BTreeMap<String, (Vec<String>, String)>,
) -> (Vec<TokenStream>, String) {
    let mut registrations = Vec::new();
    let mut dependencies = String::new();
    for (package_name, (udfs, package_dir)) in package_udfs.iter() {
        let module_ident = format_ident!("{}", package_name);
        let registration_calls = udfs.iter().map(|fn_name| {
            let func_ident = format_ident!("register_{}", fn_name);
            quote! {
                #module_ident::#func_ident();
            }
        });
        registrations.push(quote! {
            #( #registration_calls )*
        });

        let mut dep_vars: HashMap<String, String> = HashMap::new();
        dep_vars.insert("package_name".to_string(), package_name.to_string());
        dep_vars.insert("package_dir".to_string(), package_dir.to_string());
        match strfmt(r#"{package_name} = {{ path = "../{package_dir}" }}"#, &dep_vars) {
            Ok(dep) => {
                let mut deps_vars: HashMap<String, String> = HashMap::new();
                deps_vars.insert("deps".to_string(), dependencies.to_string());
                deps_vars.insert("dep".to_string(), dep);
                match strfmt("{deps}\n{dep}", &deps_vars) {
                    Ok(result) => dependencies = result,
                    _ => {}
                }
            }
            _ => {}
        }
    }
    (registrations, dependencies)
}

pub fn generate_udf_registry_and_dependencies(root_dir: &Path) -> (TokenStream, String) {
    let package_udfs = _extract_package_and_udfs(root_dir);
    let (registrations, dependencies) = _collect_udf_registration_funcs(&package_udfs);

    let register_all_funcs = quote! {
        #[udf_registry]
        pub fn register_all_funcs() {
            #( #registrations )*
        }
    };

    (register_all_funcs, dependencies)
}

const LIB_RS_TEMPLATE: &str = r#"
use gandiva_rust_udf_macro::udf_registry;

{GENERATED_UDF_REGISTRY}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_register_func() {{
        register_all_funcs();
        let registry_c_str = gandiva_rust_udf_shared::get_udf_registry();
        unsafe {{
            let registry = std::ffi::CString::from_raw(registry_c_str);
            let registry_str = registry.to_str().unwrap();
            let udf_registry: gandiva_rust_udf_shared::UdfRegistry =
                serde_json::from_str(registry_str).unwrap();
            assert!(udf_registry.functions.len() > 0);
        }}
    }}
}}
"#;

const CARGO_TOML_TEMPLATE: &str = r#"
[package]
name = "udf_core"
version = "0.1.1"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
[lib]
name = "gandiva_rust_udf"
crate-type = ["cdylib"]
path = "src/lib.rs"

[dependencies]
libc = "0.2.152"
gandiva_rust_udf_macro = {{ path = "../../gandiva_rust_udf_macros/gandiva_rust_udf_macro" }}
gandiva_rust_udf_shared = {{ path = "../../gandiva_rust_udf_macros/gandiva_rust_udf_shared" }}
{GENERATED_UDF_DEPENDENCIES}

[dev-dependencies]
serde_json = "1.0.111"

[build-dependencies]
gandiva_rust_udf_macro = {{ path = "../../gandiva_rust_udf_macros/gandiva_rust_udf_macro" }}
gandiva_rust_udf_shared = {{ path = "../../gandiva_rust_udf_macros/gandiva_rust_udf_shared" }}
strfmt = "0.2.4"
"#;

pub fn generate_udf_core_pack(root_dir: &Path) {
    let base_path = root_dir.join("udf_core");
    let lib_path = base_path.join("src").join("lib.rs");
    match fs::create_dir_all(lib_path.parent().unwrap()) {
        Ok(_) => println!("mkdir src done"),
        Err(e) => println!("failed to mkdir src: {:?}", e),
    }

    // let udf_registry_code = generate_udf_registry(base_path.parent().unwrap());
    let (udf_registry_code, udf_dependencies_code) = generate_udf_registry_and_dependencies(base_path.parent().unwrap());

    // to generate src/lib.rs
    let mut lib_rs_vars: HashMap<String, String> = HashMap::new();
    lib_rs_vars.insert(
        "GENERATED_UDF_REGISTRY".to_string(),
        udf_registry_code.to_string(),
    );
    match strfmt(LIB_RS_TEMPLATE, &lib_rs_vars) {
        Ok(result) => fs::write(&lib_path, result).expect("failed to write to src/lib.rs"),
        Err(e) => println!("failed to generate lib.rs code: {:?}", e),
    }

    // to generate Cargo.toml
    let cargo_path = base_path.join("Cargo.toml");
    let mut cargo_tml_vars: HashMap<String, String> = HashMap::new();
    cargo_tml_vars.insert("RELEASE_VER".to_string(), "0.2.0".to_string());
    cargo_tml_vars.insert("GDV_RUST_UDF_MACRO_VER".to_string(), "0.1.1".to_string());
    cargo_tml_vars.insert("GDV_RUST_UDF_SHARED_VER".to_string(), "0.1.1".to_string());
    cargo_tml_vars.insert("GENERATED_UDF_DEPENDENCIES".to_string(), udf_dependencies_code.to_string());
    match strfmt(CARGO_TOML_TEMPLATE, &cargo_tml_vars) {
        Ok(result) => fs::write(&cargo_path, result).expect("failed to write to Cargo.toml"),
        Err(e) => println!("failed to generate Cargo.toml code: {:?}", e),
    }
}