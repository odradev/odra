#[allow(unused_imports, clippy::single_component_path_imports)]
use odra_modules;

#[cfg(not(target_arch = "wasm32"))]
extern "Rust" {
    fn module_schema() -> odra::contract_def::ContractBlueprint;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let schema = unsafe { crate::module_schema() };

    if !std::path::Path::new("resources").exists() {
        std::fs::create_dir("resources").expect("Failed to create resources directory");
    }

    let module = std::env::var("ODRA_MODULE").expect("ODRA_MODULE environment variable is not set");
    let mut schema_file = std::fs::File::create(format!("resources/{}_schema.json", module))
        .expect("Failed to create schema file");
    let json = schema.as_json().expect("Failed to convert schema to JSON");
    std::io::Write::write_all(&mut schema_file, &json.into_bytes())
        .expect("Failed to write to schema file");
}
