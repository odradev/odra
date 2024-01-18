#[allow(unused_imports)]
use odra_examples;

#[cfg(not(target_arch = "wasm32"))]
extern "Rust" {
    fn module_schema() -> odra::contract_def::ContractBlueprint;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let schema = unsafe { crate::module_schema() };
    
    if !std::path::Path::new("resources").exists() {
        std::fs::create_dir("resources").unwrap();
    }
    
    let module = std::env::var("ODRA_MODULE").unwrap_or_else(|_| "".to_string());
    let mut schema_file =
        std::fs::File::create(&format!("resources/{}_schema.json", module)).unwrap();
    std::io::Write::write_all(&mut schema_file, &schema.as_json().into_bytes()).unwrap();
}
