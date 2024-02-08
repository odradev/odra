use benchmark::benchmark::BenchmarkHostRef;
use odra::host::{Deployer, NoArgs};
use std::fs;
use std::path::PathBuf;

pub fn main() {
    println!("Running benchmark...");
    let env = odra_test::env();

    let _ = BenchmarkHostRef::deploy(&env, NoArgs);

    // convert gas_report to json and dump it into a file
    let gas_report = env.gas_report();
    let gas_report_json = serde_json::to_string(&gas_report).unwrap();
    let path = PathBuf::from("gas_report.json");
    fs::write(path, gas_report_json).unwrap();
}
