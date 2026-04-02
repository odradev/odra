const REVERT_MESSAGE_PREFIX: &str = "Revert: ExecutionError";
const CONTRACT_PREFIX: &str = "Contract(ContractPackageHash";
const USER_ERROR_PREFIX: &str = "UserError { code: ";
const EXEC_PARTS_PREFIX: &str = "exec_parts::execute";

pub fn set_odra_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let panic_message = if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "<unknown>".into());

        // Print the panic message if an assertion failed
        if panic_message.contains("assertion") {
            eprintln!("💥 {panic_message}");
            eprintln!("    ↳ at {location}");
        }

        if let Some(contract) = extract_contract_address(&panic_message) {
            eprintln!("{contract}");
        }

        if panic_message.starts_with(REVERT_MESSAGE_PREFIX) {
            if let Some(error_name) = extract_error_name(&panic_message) {
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
