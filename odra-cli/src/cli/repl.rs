//! Interactive REPL mode for the Odra CLI.
//!
//! Reads commands line by line, keeping the `HostEnv` and the deployed-contracts container warm
//! across calls. Parse errors and failing commands are reported and the prompt returns, so a bad
//! line can never kill the session.

use std::path::PathBuf;

use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;

use super::completer::ReplHelper;
use crate::cmd::{DEPLOY_SUBCOMMAND, REPL_SUBCOMMAND};
use crate::container::ContractStorageSource;
use crate::{ContractProvider, DeployedContractsContainer, OdraCli};

const HISTORY_FILE: &str = ".odra_cli_history";
const DEFAULT_PROMPT: &str = "odra> ";

/// Runs the interactive REPL until the user exits (`exit`/`quit`/Ctrl-D).
///
/// Note: the per-command `--contracts-toml` flag is meaningless mid-session — the container is fixed
/// for the lifetime of the REPL (chosen when the `repl` subcommand set it up), so the flag is parsed
/// but ignored.
pub(super) fn run(cli: &OdraCli, container: &mut DeployedContractsContainer) -> anyhow::Result<()> {
    // Complete against the same command tree the REPL parses against, with `repl` excluded so it
    // can't be suggested mid-session. The built-ins are handled by this loop, not by clap.
    let mut editor: Editor<ReplHelper, FileHistory> = Editor::new()?;
    let helper = ReplHelper::new(
        cli.main_cmd.to_command(&[REPL_SUBCOMMAND]),
        &["help", "exit", "quit"]
    );
    editor.set_helper(Some(helper));

    let history_path = history_path();
    // A missing history file just means we have nothing to load yet.
    let _ = editor.load_history(&history_path);

    print_banner(cli);
    let prompt = prompt();

    loop {
        // Refreshed every prompt so it reflects contracts deployed mid-session.
        print_status_line(cli, container);
        match editor.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                // Built-ins handled before clap so they work without a registered subcommand.
                match line {
                    "exit" | "quit" => break,
                    "help" => {
                        // Exclude `repl` — it isn't a valid command from within a session.
                        let mut help_cmd = cli.main_cmd.to_command(&[REPL_SUBCOMMAND]);
                        let _ = help_cmd.print_help();
                        println!();
                        continue;
                    }
                    _ => {}
                }

                let _ = editor.add_history_entry(line);

                let tokens = match shlex::split(line) {
                    Some(tokens) => tokens,
                    None => {
                        prettycli::error("Could not parse input: unbalanced quotes.");
                        continue;
                    }
                };
                let mut argv = vec!["odra-cli".to_string()];
                argv.extend(tokens);

                // Exclude `repl` so it can't be invoked recursively from inside a session.
                match cli
                    .main_cmd
                    .try_get_matches_from_excluding(argv, &[REPL_SUBCOMMAND])
                {
                    Ok((cmd, args, _path)) => {
                        if let Err(err) = cli.dispatch(&cmd, &args, container) {
                            prettycli::error(&format!("{err:#}"));
                            continue;
                        }
                        // Make freshly deployed contracts callable in the same session.
                        if cmd == DEPLOY_SUBCOMMAND {
                            if let Err(err) = cli
                                .register_deployed_contracts(container, &cli.default_contract_path)
                            {
                                prettycli::error(&format!("{err:#}"));
                            }
                        }
                    }
                    // Renders real parse errors and `--help`/`-h`/subcommand help alike.
                    Err(clap_err) => {
                        let _ = clap_err.print();
                    }
                }
            }
            // Ctrl-C cancels the current line.
            Err(ReadlineError::Interrupted) => continue,
            // Ctrl-D exits the session.
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                prettycli::error(&format!("Input error: {err}"));
                break;
            }
        }
    }

    let _ = editor.save_history(&history_path);
    Ok(())
}

/// Prints a short banner showing which network and account the session is bound to.
fn print_banner(cli: &OdraCli) {
    let caller = cli.host_env.caller();
    prettycli::info(&format!("Odra CLI interactive session — {}", chain_label()));
    prettycli::info(&format!("Caller: {}", caller.to_string()));
    prettycli::info("Type `help` for available commands, `exit` or Ctrl-D to quit.");
}

/// Prints the one-line session status shown above each prompt — network, caller and the number of
/// deployed contracts the session currently knows about. Dimmed so it reads as chrome rather than
/// command output.
fn print_status_line(cli: &OdraCli, container: &DeployedContractsContainer) {
    let network = match std::env::var("ODRA_CASPER_LIVENET_CHAIN_NAME") {
        Ok(name) if !name.is_empty() => name,
        _ => "no chain".to_string()
    };
    let caller = short_address(&cli.host_env.caller().to_string());
    let file = contracts_file_name(container);
    let count = container.all_contracts().len();
    let contracts = format!(
        "{count} {}",
        if count == 1 { "contract" } else { "contracts" }
    );

    // `\x1b[2m` = dim, `\x1b[0m` = reset.
    println!("\x1b[2m⬡ {network}  ·  {caller}  ·  {file} ({contracts})\x1b[0m");
}

/// The bare file name of the contracts source (e.g. `casper-net-1-contracts.toml`), or `memory`
/// for a non-file-backed container. Just the name keeps the status line compact.
fn contracts_file_name(container: &DeployedContractsContainer) -> String {
    match container.source() {
        ContractStorageSource::File { path } => path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned()),
        ContractStorageSource::Memory => "memory".to_string()
    }
}

/// Shortens a long address to `head…tail` for compact display, leaving short strings untouched.
fn short_address(addr: &str) -> String {
    let chars: Vec<char> = addr.chars().collect();
    if chars.len() <= 24 {
        return addr.to_string();
    }
    let head: String = chars[..16].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}…{tail}")
}

/// The prompt, derived from the chain name when available.
fn prompt() -> String {
    match std::env::var("ODRA_CASPER_LIVENET_CHAIN_NAME") {
        Ok(name) if !name.is_empty() => format!("{name}> "),
        _ => DEFAULT_PROMPT.to_string()
    }
}

/// Human-readable chain name for the banner.
fn chain_label() -> String {
    match std::env::var("ODRA_CASPER_LIVENET_CHAIN_NAME") {
        Ok(name) if !name.is_empty() => format!("chain `{name}`"),
        _ => "unknown chain".to_string()
    }
}

/// History file location: `$HOME/.odra_cli_history`, falling back to the current directory.
fn history_path() -> PathBuf {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() => PathBuf::from(home).join(HISTORY_FILE),
        _ => PathBuf::from(HISTORY_FILE)
    }
}

#[cfg(test)]
mod tests {
    use super::{contracts_file_name, short_address};
    use crate::test_utils;

    #[test]
    fn short_address_truncates_long_addresses() {
        let addr = "account-hash-2a6da1c25eb66de30ad01e65fe37d111f37813db12487998093c628a153a7961";
        let short = short_address(addr);
        assert_eq!(short, "account-hash-2a6…7961");
        assert!(short.len() < addr.len());
    }

    #[test]
    fn short_address_leaves_short_strings_untouched() {
        assert_eq!(short_address("no chain"), "no chain");
    }

    #[test]
    fn contracts_file_name_for_memory_container() {
        // The mock container is memory-backed, so there is no file name.
        let container = test_utils::mock_contracts_container();
        assert_eq!(contracts_file_name(&container), "memory");
    }
}
