//! Interactive REPL mode for the Odra CLI.
//!
//! Reads commands line by line, keeping the `HostEnv` and the deployed-contracts container warm
//! across calls. Parse errors and failing commands are reported and the prompt returns, so a bad
//! line can never kill the session.

use std::path::PathBuf;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::cmd::{DEPLOY_SUBCOMMAND, REPL_SUBCOMMAND};
use crate::{DeployedContractsContainer, OdraCli};

const HISTORY_FILE: &str = ".odra_cli_history";
const DEFAULT_PROMPT: &str = "odra> ";

/// Runs the interactive REPL until the user exits (`exit`/`quit`/Ctrl-D).
///
/// Note: the per-command `--contracts-toml` flag is meaningless mid-session — the container is fixed
/// for the lifetime of the REPL (chosen when the `repl` subcommand set it up), so the flag is parsed
/// but ignored.
pub(super) fn run(cli: &OdraCli, container: &mut DeployedContractsContainer) -> anyhow::Result<()> {
    let mut editor = DefaultEditor::new()?;
    let history_path = history_path();
    // A missing history file just means we have nothing to load yet.
    let _ = editor.load_history(&history_path);

    print_banner(cli);
    let prompt = prompt();

    loop {
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
