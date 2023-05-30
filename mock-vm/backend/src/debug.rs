use std::{backtrace::Backtrace, panic::PanicInfo};

use colored::Colorize;
use odra_types::ExecutionError;

use crate::callstack::CallstackElement;

const ERROR_PREFIX: &str = "Contract panicked!";

pub fn print_first_n_frames(bt: &Backtrace, n: usize) {
    let frames = format!("{}", bt);
    let frames = frames.lines().skip(1); // Skip the first line, which is "Backtrace:"
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

pub fn format_panic_message(error: &ExecutionError, callstack_elem: &CallstackElement) -> String {
    match callstack_elem {
        crate::callstack::CallstackElement::Account(_) => format!("Panic {:?}", error),
        crate::callstack::CallstackElement::Entrypoint(ep) => {
            let address = format!("{:?}", ep.address);
            let error = format!("{:?}", error);
            format!(
                "{}\naddress: {}\nentrypoint: {}\nerror: {}",
                ERROR_PREFIX.bold().red(),
                address.bold().purple(),
                ep.entrypoint.bold().purple(),
                error.bold().purple()
            )
        }
    }
}
