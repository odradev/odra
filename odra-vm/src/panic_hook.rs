const REVERT_MESSAGE_PREFIX: &str = "Revert: ExecutionError";
const CONTRACT_PREFIX: &str = "Contract(ContractPackageHash";
const USER_ERROR_PREFIX: &str = "UserError { code: ";
const EXEC_PARTS_PREFIX: &str = "exec_parts::execute";

pub fn set_odra_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let panic_message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "Unknown panic"
        };

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "<unknown>".into());
        if let Some(contract) = extract_contract_address(panic_message) {
            eprintln!("{contract}");
        }
        if panic_message.starts_with(REVERT_MESSAGE_PREFIX) {
            if let Some(error_name) = extract_error_name(panic_message) {
                eprintln!("💣 {error_name}");
            } else {
                eprintln!("💣 {panic_message} at {location}");
            }
        }
        // Find the first symbol that contains `exec_parts::execute` in its name
        // to identify the place where the panic occurred in the contract code.
        let backtrace = backtrace::Backtrace::new();
        let mut prev_symbol: Option<backtrace::BacktraceSymbol> = None;
        for frame in backtrace.frames() {
            for symbol in frame.symbols() {
                match (symbol.name(), symbol.filename(), symbol.lineno()) {
                    (Some(name), Some(filename), Some(lineno)) => {
                        let name = name.to_string();
                        if name.contains(EXEC_PARTS_PREFIX) {
                            if let Some(prev) = prev_symbol {
                                match (prev.name(), prev.filename(), prev.lineno()) {
                                    (Some(prev_name), Some(prev_filename), Some(prev_lineno)) => {
                                        let prev_name = prev_name.to_string();
                                        let prev_name = prev_name
                                            .rfind("::")
                                            .map_or(prev_name.clone(), |pos| {
                                                prev_name[..pos].to_string()
                                            });
                                        eprintln!("  ↳ {prev_name}");
                                        eprintln!(
                                            "    ↳ at {}:{prev_lineno}",
                                            prev_filename.to_string_lossy()
                                        );
                                    }
                                    _ => {} // no-op
                                }
                                return;
                            }
                        }
                        prev_symbol = Some(symbol.clone());
                    }
                    _ => {} // no-op
                }
            }
        }
    }));
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
