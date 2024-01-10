use odra::contract_def::ContractBlueprint;

extern "Rust" {
    fn module_schema() -> ContractBlueprint;
}

fn main() {
    let schema = unsafe { module_schema() };
    println!("{:#?}", schema);
}
