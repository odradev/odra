#![doc = "Binary for building schema definitions from odra contracts."]
#[allow(unused_imports, clippy::single_component_path_imports)]
use odra_modules;

#[cfg(not(target_arch = "wasm32"))]
extern "Rust" {
    fn module_schema() -> odra::contract_def::ContractBlueprint;
    fn casper_contract_schema() -> odra::schema::casper_contract_schema::ContractSchema;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if !std::path::Path::new("resources").exists() {
        std::fs::create_dir("resources").expect("Failed to create resources directory");
    }

    let module = std::env::var("ODRA_MODULE").expect("ODRA_MODULE environment variable is not set");
    let module = to_snake_case(&module);

    let contract_schema = unsafe { crate::casper_contract_schema() };
    let module_schema = unsafe { crate::module_schema() };

    write_schema_file(
        format!("resources/{}_contract_schema.json", module),
        contract_schema
            .as_json()
            .expect("Failed to convert schema to JSON")
    );

    write_schema_file(
        format!("resources/{}_schema.json", module),
        module_schema
            .as_json()
            .expect("Failed to convert schema to JSON")
    );
}

fn write_schema_file(filename: String, json: String) {
    let mut schema_file = std::fs::File::create(filename).expect("Failed to create schema file");

    std::io::Write::write_all(&mut schema_file, &json.into_bytes())
        .expect("Failed to write to schema file");
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut is_first = true;

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if !is_first {
                if let Some(next) = chars.peek() {
                    if next.is_lowercase() {
                        result.push('_');
                    }
                }
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
        is_first = false;
    }

    result
}
