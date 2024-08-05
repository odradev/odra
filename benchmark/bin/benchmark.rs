use benchmark::benchmark::{Benchmark, StructVariable};
use odra::host::{Deployer, HostRef, NoArgs};
use odra_test::env;
use std::fs;
use std::path::PathBuf;

pub fn main() {
    println!("Running benchmark...");
    let env = env();

    let mut contract = Benchmark::deploy(&env, NoArgs);
    // Var
    contract.set_variable(true);
    assert!(contract.get_variable());

    // Struct in Var
    contract.set_struct_variable(struct_variable());
    assert_eq!(contract.get_struct_variable(), struct_variable());

    // Mapping
    contract.set_mapping(42, true);
    assert!(contract.get_mapping(42));

    // List
    contract.push_list(42);
    assert_eq!(contract.get_list(0), 42);

    // Submodule
    contract.init_submodule();
    assert_eq!(contract.call_submodule(), 1_000_000_000.into());

    // Payable
    contract.with_tokens(1_000_000_000.into()).call_payable();
    contract.transfer_back(1_000_000_000.into());

    // Events
    contract.emit_event();

    // Named key
    contract.set_named_key("Hello, world!".to_string());
    assert_eq!(contract.get_named_key(), "Hello, world!".to_string());

    // Dictionary
    contract.set_dictionary("My key".to_string(), 42.into());
    assert_eq!(contract.get_dictionary("My key".to_string()), 42.into());

    // convert gas_report to json and dump it into a file
    let gas_report = env.gas_report();
    let gas_report_json = serde_json::to_string(&gas_report).unwrap();
    let path = PathBuf::from("gas_report.json");
    fs::write(path, gas_report_json).unwrap();
}

fn struct_variable() -> StructVariable {
    StructVariable {
        yes_or_no: true,
        number: 42,
        title: "Hello, world!".to_string()
    }
}
