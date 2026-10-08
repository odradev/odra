use std::cell::Cell;
use std::sync::Once;

const REVERT_PREFIX: &str = "Revert: ";
const REVERT_MESSAGE_PREFIX: &str = "Revert: ExecutionError";
const CONTRACT_PREFIX: &str = "Contract(ContractPackageHash";
const USER_ERROR_PREFIX: &str = "UserError { code: ";
const EXEC_PARTS_PREFIX: &str = "exec_parts::execute";

thread_local! {
    static REVERTING: Cell<bool> = const { Cell::new(false) };
}

/// Marks the next panic on this thread as an Odra revert. Called by the VM right before it panics
/// to unwind a reverted call.
pub fn mark_revert() {
    REVERTING.with(|r| r.set(true));
}

/// Installs the Odra panic hook, once per process.
///
/// Reverts are printed in a compact form. Any other panic goes to the hook that was installed
/// before, so it is reported as usual (the default hook prints the message, location and backtrace).
pub fn set_odra_panic_hook() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let is_revert = REVERTING.with(|r| r.replace(false));
            let panic_message = panic_info
                .payload()
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_default();
            if is_revert && panic_message.starts_with(REVERT_PREFIX) {
                print_revert(&panic_message);
            } else {
                previous_hook(panic_info);
            }
        }));
    });
}

fn print_revert(panic_message: &str) {
    if let Some(contract) = extract_contract_address(panic_message) {
        eprintln!("{contract}");
    }

    if panic_message.starts_with(REVERT_MESSAGE_PREFIX) {
        if let Some(error_name) = extract_error_name(panic_message) {
            eprintln!("💣 {error_name}");
        } else {
            eprintln!("💣 {panic_message}");
        }

        // Find the first symbol that contains `exec_parts::execute` in its name
        // to identify the place where the panic occurred in the contract code.
        let backtrace = backtrace::Backtrace::new();
        let mut prev_symbol: Option<backtrace::BacktraceSymbol> = None;
        let mut should_print = false;
        for frame in backtrace.frames() {
            for symbol in frame.symbols() {
                if let (Some(name), Some(_filename), Some(_lineno)) =
                    (symbol.name(), symbol.filename(), symbol.lineno())
                {
                    let name = name.to_string();
                    if should_print {
                        if !name.contains("HostRef::") {
                            print_symbol_with_location(symbol);
                            return;
                        }
                        should_print = false;
                    }
                    if name.contains(EXEC_PARTS_PREFIX) {
                        if let Some(prev) = &prev_symbol {
                            print_symbol_with_location(prev);
                        }
                    }
                    if name.contains("HostRef::") && prev_symbol.is_some() {
                        should_print = true;
                    }
                    prev_symbol = Some(symbol.clone());
                }
            }
        }
    }
}

fn extract_error_name(panic_message: &str) -> Option<String> {
    let parts: Vec<&str> = panic_message.split(USER_ERROR_PREFIX).collect();
    if parts.len() > 1 {
        let code_and_message = parts[1].split(" }").next().unwrap_or("");
        let code_parts: Vec<&str> = code_and_message.split(", message: ").collect();
        if code_parts.len() > 1 {
            let code = code_parts[0].trim();
            let message = code_parts[1].trim_matches('"');
            return Some(format!("{}({})", message, code));
        }
    }
    None
}

fn extract_contract_address(panic_message: &str) -> Option<String> {
    let contract_idx = panic_message.find(CONTRACT_PREFIX)?;
    let contract_part = &panic_message[contract_idx..];
    Some(contract_part.to_string())
}

fn print_symbol_with_location(symbol: &backtrace::BacktraceSymbol) {
    if let (Some(name), Some(filename), Some(lineno)) =
        (symbol.name(), symbol.filename(), symbol.lineno())
    {
        let name = name.to_string();
        let name = name
            .rfind("::")
            .map_or(name.clone(), |pos| name[..pos].to_string());
        eprintln!("  ↳ {name}");
        eprintln!("    ↳ at {}:{lineno}", filename.to_string_lossy());
    }
}
