use crate::type_mapping::map_type;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::fs;
use std::path::Path;
use syn::{visit::Visit, File, FnArg, ItemFn};
use toml::Value;
use walkdir::WalkDir;

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

fn _get_cargo_package_name(path: &Path) -> Option<String> {
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
                return Some(name.to_string());
            }
        }
    }
    None
}

// use an ordered map <package_name, Vec<String>> to store all packages with their UDFs within
// the map is ordered so that the generated code is deterministic
fn _extract_package_and_udfs(root_dir: &Path) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut package_udfs = std::collections::BTreeMap::new();
    for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let package_name = _get_cargo_package_name(path);
        if package_name.is_some() {
            let mut collector = UdfCollector {
                udf_registration_funcs: Vec::new(),
            };
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let sub_path = entry.path();
                if sub_path.is_file()
                    && sub_path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs")
                {
                    let content = fs::read_to_string(sub_path).expect("Failed to read file");
                    let syntax_tree: File = syn::parse_str(&content).expect("Failed to parse file");
                    syn::visit::visit_file(&mut collector, &syntax_tree);
                }
            }
            package_udfs.insert(package_name.unwrap(), collector.udf_registration_funcs);
        }
    }
    package_udfs
}

fn _collect_udf_registration_funcs(
    package_udfs: &std::collections::BTreeMap<String, Vec<String>>,
) -> Vec<TokenStream> {
    let mut registrations = Vec::new();
    for (package_name, udfs) in package_udfs.iter() {
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
    }
    registrations
}

pub fn generate_udf_registry(root_dir: &Path) -> TokenStream {
    let package_udfs = _extract_package_and_udfs(root_dir);
    let registrations = _collect_udf_registration_funcs(&package_udfs);

    let register_all_funcs = quote! {
        #[udf_registry]
        pub fn register_all_funcs() {
            #( #registrations )*
        }
    };

    register_all_funcs
}
