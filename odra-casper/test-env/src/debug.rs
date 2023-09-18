use std::{backtrace::Backtrace, panic::PanicInfo};

use colored::Colorize;
use odra_types::{OdraError, casper_types::ContractPackageHash};

const ERROR_PREFIX: &str = "Contract panicked!";

pub fn print_first_n_frames(bt: &Backtrace, n: usize) {
    let frames = format!("{}", bt);
    let frames = frames.lines();
    for (i, line) in frames.enumerate() {
        if i >= n {
            break;
        }
        println!("{}", line.bold());
    }
}

pub fn print_panic_error(info: &PanicInfo) {
    let string = format!("{}", info);
    if let (Some(start), Some(end)) = (string.find('\''), string.rfind('\'')) {
        if start < end {
            println!("{}", &string[start + 1..end]);
        } else {
            println!("{}", string);
        }
    } else {
        println!("{}", string);
    }
}

pub fn format_panic_message(
    error: &OdraError,
    entrypoint: &str,
    contract_package_hash: ContractPackageHash
) -> String {
    match error {
        OdraError::ExecutionError(error) => {
            let address = format!("{:?}", contract_package_hash);
            let error = format!("{:?}", error);
            format!(
                "{}\naddress: {}\nentrypoint: {}\nerror: {}",
                ERROR_PREFIX.bold().red(),
                address.bold().purple(),
                entrypoint.bold().purple(),
                error.bold().purple()
            )
        }
        OdraError::VmError(_) => format!("Panic {:?}", error)
    }
}
