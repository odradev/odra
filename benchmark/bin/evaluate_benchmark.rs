use odra::{DeployReport, GasReport};
use std::fs;
use std::ops::{Div, Mul};
use std::path::PathBuf;
use std::process::exit;

pub fn main() {
    println!("Evaluating benchmark...");
    let current_gas_report_path = PathBuf::from("gas_report.json");
    let base_gas_report_path = PathBuf::from("base/gas_report.json");

    let current_gas_report = load_gas_report(&current_gas_report_path);
    let base_gas_report = load_gas_report(&base_gas_report_path);

    exit(compare_gas_reports(&current_gas_report, &base_gas_report));
}

fn load_gas_report(file: &PathBuf) -> GasReport {
    let gas_report_json = fs::read(file).unwrap();
    serde_json::from_str(String::from_utf8(gas_report_json).unwrap().as_str()).unwrap()
}

fn compare_gas_reports(current_gas_report: &GasReport, base_gas_report: &GasReport) -> i32 {
    let mut exit_code = 0;
    let mut report: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut unchanged: Vec<String> = Vec::new();
    for (current_report, base_report) in current_gas_report.iter().zip(base_gas_report) {
        match (current_report, base_report) {
            (
                DeployReport::WasmDeploy {
                    gas: current_gas,
                    file_name: current_file_name
                },
                DeployReport::WasmDeploy {
                    gas: target_gas,
                    file_name: base_file_name
                }
            ) => {
                if current_file_name == base_file_name {
                    if current_gas != target_gas {
                        let diff = current_gas.abs_diff(*target_gas);
                        let percentage = (diff.as_u64() as f64)
                            .div(target_gas.as_u64() as f64)
                            .mul(100.0);
                        let diff = (diff.as_u64() as f64).div(1_000_000_000.0);
                        let symbol = if current_gas > target_gas {
                            ":red_circle: +"
                        } else {
                            ":green_circle: -"
                        };
                        report.push(format!(
                            "| Wasm Deploy | Filename: {} | {}{} CSPR ({:.2}%)</span> |",
                            current_file_name, symbol, diff, percentage
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
                        current_file_name, base_file_name
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
                    call_def: base_call_def,
                    ..
                }
            ) => {
                if current_call_def == base_call_def {
                    if current_gas != target_gas {
                        let diff = current_gas.abs_diff(*target_gas);
                        let percentage = (diff.as_u64() as f64)
                            .div(target_gas.as_u64() as f64)
                            .mul(100.0);
                        let diff = (diff.as_u64() as f64).div(1_000_000_000.0);
                        let symbol = if current_gas > target_gas {
                            ":red_circle: +"
                        } else {
                            ":green_circle: -"
                        };
                        report.push(format!(
                            "| Contract Call | Entry point: {} | {}{} CSPR ({:.2}%)</span> |",
                            current_call_def.entry_point(),
                            symbol,
                            diff,
                            percentage
                        ));
                    } else {
                        unchanged.push(format!(
                            "Contract call: {}, Cost: {}",
                            current_call_def.entry_point(),
                            current_gas
                        ));
                    }
                } else {
                    errors.push(format!(
                        "Deploy changed:\n\
                        | Current | Base | \n\
                        | --- | --- | \n\
                        |{}|{}|",
                        current_report, base_report
                    ));
                }
            }

            _ => {}
        }
    }

    if !report.is_empty() {
        report.insert(0, "### Benchmark report".to_string());
        report.insert(1, "| Action | Details | Gas diff |".to_string());
        report.insert(2, "| --- | --- | --- |".to_string());
        report.iter().for_each(|message| {
            println!("{}", message);
        });

        exit_code += 1;
    } else {
        println!("*No differences found in the benchmarks.*");
    }

    if !errors.is_empty() {
        errors.insert(0, "### Errors".to_string());
        errors.iter().for_each(|error| {
            println!("- {}", error);
        });

        exit_code += 2;
    }

    let report = format!("{}\n{}", report.join("\n"), errors.join("\n"),);

    // Write report to file
    fs::write("benchmark_report.txt", &report).unwrap();

    // Write full report to file
    let full_report = format!(
        "{}\n{}\n{}",
        report,
        if unchanged.is_empty() {
            "### Unchanged"
        } else {
            ""
        },
        unchanged.join("\n"),
    );
    fs::write("benchmark_full_report.txt", full_report).unwrap();

    exit_code
}
