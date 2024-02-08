use odra::{DeployReport, GasReport};
use std::fs;
use std::path::PathBuf;
use std::process::exit;

pub fn main() {
    println!("Evaluating benchmark...");
    let current_gas_report_path = PathBuf::from("gas_report.json");
    let target_gas_report_path = PathBuf::from("target_gas_report.json");

    let current_gas_report = load_gas_report(&current_gas_report_path);
    let target_gas_report = load_gas_report(&target_gas_report_path);

    exit(compare_gas_reports(&current_gas_report, &target_gas_report));
}

fn load_gas_report(file: &PathBuf) -> GasReport {
    let gas_report_json = fs::read(file).unwrap();
    serde_json::from_str(String::from_utf8(gas_report_json).unwrap().as_str()).unwrap()
}

fn compare_gas_reports(current_gas_report: &GasReport, target_gas_report: &GasReport) -> i32 {
    let mut exit_code = 0;
    let mut report: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut unchanged: Vec<String> = Vec::new();
    for (current_report, target_report) in current_gas_report.iter().zip(target_gas_report) {
        match (current_report, target_report) {
            (
                DeployReport::WasmDeploy {
                    gas: current_gas,
                    file_name: current_file_name
                },
                DeployReport::WasmDeploy {
                    gas: target_gas,
                    file_name: target_file_name
                }
            ) => {
                if current_file_name == target_file_name {
                    if current_gas != target_gas {
                        let diff = current_gas.abs_diff(*target_gas);
                        let symbol = if current_gas > target_gas {
                            ":red_circle:+"
                        } else {
                            ":green_circle:-"
                        };
                        report.push(format!(
                            "| Wasm Deploy | Filename: {} | {}{}</span> |",
                            current_file_name, symbol, diff
                        ));
                    } else {
                        unchanged.push(format!(
                            "Wasm deploy: {}, Cost: {}",
                            current_file_name, current_gas
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Wasm filename changed: current: {}, target: {}",
                        current_file_name, target_file_name
                    ));
                }
            }
            (
                DeployReport::ContractCall {
                    gas: current_gas,
                    call_def: current_call_def,
                    ..
                },
                DeployReport::ContractCall {
                    gas: target_gas,
                    call_def: target_call_def,
                    ..
                }
            ) => if current_call_def == target_call_def {
                if current_gas != target_gas {
                    let diff = current_gas.abs_diff(*target_gas);
                    let symbol = if current_gas > target_gas {
                        ":red_circle:+"
                    } else {
                        ":green_circle:-"
                    };
                    report.push(
                        format!(
                            "| Contract Call | Entry point: {} | {}{}</span> |",
                            current_call_def.entry_point(),
                            symbol,
                            diff
                        )
                    );
                } else {
                    unchanged.push(format!(
                        "Contract call: {}, Cost: {}",
                        current_call_def.entry_point(),
                        current_gas
                    ));
                }
            } else {
                errors.push(format!(
                    "Contract call changed: current: {:?}, target: {:?}",
                    current_report, target_report
                ));
            },

            _ => {}
        }
    }

    if !report.is_empty() {
        println!("Benchmark report:");
        println!("| Action | Details | Gas diff |");
        println!("| --- | --- | --- |");
        report.iter().for_each(|message| {
            println!("{}", message);
        });

        println!("\nTo see the full report, check the artifacts section.");
        exit_code += 1;
    } else {
        println!("No differences found in the benchmark.");
    }

    if !errors.is_empty() {
        println!("Errors:");
        errors.iter().for_each(|error| {
            println!("{}", error);
        });

        exit_code += 2;
    }

    // Write full report to file
    let full_report = format!(
        "Benchmark report: \n{}\nErrors: \n{}\nUnchanged: \n{}",
        report.join("\n"),
        errors.join("\n"),
        unchanged.join("\n")
    );
    fs::write("benchmark_report.txt", full_report).unwrap();

    exit_code
}
