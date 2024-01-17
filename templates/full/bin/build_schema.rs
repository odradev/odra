#[cfg(not(target_arch = "wasm32"))]
extern "Rust" {
    fn module_schema() -> odra::contract_def::ContractBlueprint;
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let schema = unsafe { module_schema() };
    println!("{:#?}", schema);
}
