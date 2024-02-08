use odra::{DeployReport, GasReport};
use std::ops::{Div, Mul};
use std::path::PathBuf;
use std::process::exit;
use std::{env, fs};

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!(
            "Usage: {} <current_gas_report_path> <base_gas_report_path>",
            args[0]
        );
        exit(1);
    }

    println!("Evaluating benchmark...");
    let current_gas_report_path = PathBuf::from(&args[1]);
    let base_gas_report_path = PathBuf::from(&args[2]);
    let current_gas_report = load_gas_report(&current_gas_report_path);
    let base_gas_report = load_gas_report(&base_gas_report_path);

    exit(compare_gas_reports(&current_gas_report, &base_gas_report));
}

fn load_gas_report(file: &PathBuf) -> GasReport {
    let gas_report_json =
        fs::read(file).unwrap_or_else(|_| panic!("Failed to read file: {:?}", file));
    serde_json::from_str(
        String::from_utf8(gas_report_json)
            .unwrap_or_else(|_| panic!("Failed to parse file {:?}", file))
            .as_str()
    )
    .unwrap_or_else(|_| panic!("Failed to parse json: {:?}", file))
}

fn compare_gas_reports(current_gas_report: &GasReport, base_gas_report: &GasReport) -> i32 {
    let mut exit_code = 0;
    let mut report: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut unchanged: Vec<String> = Vec::new();
    for (current_report, base_report) in current_gas_report.iter().zip(base_gas_report) {
        let (report_line, error_line, unchanged_line) =
            compare_reports(current_report, base_report);
        if let Some(line) = report_line {
            report.push(line);
        }
        if let Some(line) = error_line {
            errors.push(line);
        }
        if let Some(line) = unchanged_line {
            unchanged.push(line);
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
        println!("Errors:");
        errors.iter().for_each(|error| {
            println!("- {}", error);
        });

        exit_code += 2;
    }

    let report = format!("{}\n{}", report.join("\n"), errors.join("\n"),);

    // Write report to file
    fs::write("benchmark_report.txt", &report).expect("Failed to write report to file");

    // Write full report to file
    let full_report = format!(
        "{}\n{}\n{}",
        report,
        if !unchanged.is_empty() {
            "### Unchanged"
        } else {
            ""
        },
        unchanged.join("\n"),
    );
    fs::write("benchmark_full_report.txt", full_report)
        .expect("Failed to write full report to file");

    exit_code
}

fn compare_reports(
    current_report: &DeployReport,
    base_report: &DeployReport
) -> (Option<String>, Option<String>, Option<String>) {
    let mut report: Option<String> = None;
    let mut errors: Option<String> = None;
    let mut unchanged: Option<String> = None;

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
                    report = Some(
                        format!(
                            "| Wasm Deploy | Filename: {} | {}{} CSPR ({:.2}%)</span> |",
                            current_file_name, symbol, diff, percentage
                        )
                        .to_string()
                    );
                } else {
                    unchanged = Some(format!(
                        "Wasm deploy: {}, Cost: {}",
                        current_file_name, current_gas
                    ));
                }
            } else {
                errors = Some(format!(
                    "Wasm filename changed:\n\
                        | Current | Base | \n\
                        | --- | --- | \n\
                        |{}|{}|",
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
                    report = Some(format!(
                        "| Contract Call | Entry point: {} | {}{} CSPR ({:.2}%)</span> |",
                        current_call_def.entry_point(),
                        symbol,
                        diff,
                        percentage
                    ));
                } else {
                    unchanged = Some(format!(
                        "Contract call: {}, Cost: {}",
                        current_call_def.entry_point(),
                        current_gas
                    ));
                }
            } else {
                errors = Some(format!(
                    "Deploy changed:\n\
                        | Current | Base | \n\
                        | --- | --- | \n\
                        |{}|{}|",
                    current_report, base_report
                ));
            }
        }
        _ => {
            errors = Some(format!(
                "Different deploy types:\n\
                    | Current | Base | \n\
                    | --- | --- | \n\
                    |{}|{}|",
                current_report, base_report
            ));
        }
    }
    (report, errors, unchanged)
}
