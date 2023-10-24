#[allow(unused_imports)]
use examples2;
use odra2::types::contract_def::ContractBlueprint2;

extern "Rust" {
    fn module_schema() -> ContractBlueprint2;
}

fn main() {
    let schema = unsafe { module_schema() };
    println!("{:#?}", schema);
}
