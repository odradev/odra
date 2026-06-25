//! Interactive setup for the livenet host environment.
//!
//! The livenet `HostEnv` needs four `ODRA_CASPER_LIVENET_*` settings: the node RPC address, the
//! chain name, the events URL and the secret-key path. Previously a single missing value aborted
//! the whole CLI before the user could do anything about it. Instead we prompt for the missing (or
//! unloadable) value at runtime, set it in the process environment and retry — so a fresh checkout
//! can be configured without first hand-editing a `.env` file.
//!
//! Prompting only makes sense with a human at the keyboard. On a non-interactive stdin (CI, pipes)
//! we keep the original fail-fast behaviour: print what is missing and exit non-zero.
//!
//! Once the configuration is complete we offer to persist the values the user just typed to a
//! `.env` file in the current directory, so the next run starts without any prompting.

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use odra::host::HostEnv;
use odra_casper_livenet_env::LivenetError;

const ENV_NODE_ADDRESS: &str = "ODRA_CASPER_LIVENET_NODE_ADDRESS";
const ENV_CHAIN_NAME: &str = "ODRA_CASPER_LIVENET_CHAIN_NAME";
const ENV_EVENTS_URL: &str = "ODRA_CASPER_LIVENET_EVENTS_URL";
const ENV_SECRET_KEY_PATH: &str = "ODRA_CASPER_LIVENET_SECRET_KEY_PATH";

const DOCS_URL: &str = "https://odra.dev/docs/backends/livenet#setup";

/// Builds the livenet host environment, prompting for any missing configuration on a TTY.
///
/// Loops over [`env_safe`](odra_casper_livenet_env::env_safe): on a fixable configuration error it
/// asks for the offending value, sets it in the environment and tries again. Values are validated
/// the same way as on startup, so a wrong secret-key path simply re-prompts. Anything that can't be
/// fixed by a prompt — or an empty answer / missing TTY — exits the process.
pub(super) fn create_host_env() -> HostEnv {
    let mut intro_shown = false;
    // Values the user typed this run, kept in entry order so we can offer to save them.
    let mut entered: Vec<(String, String)> = Vec::new();
    loop {
        let err = match odra_casper_livenet_env::env_safe() {
            Ok(env) => {
                offer_to_save(&entered);
                return env;
            }
            Err(err) => err
        };

        // Work out which variable to (re)ask for; bail on anything a prompt can't fix.
        let var = match &err {
            LivenetError::EnvVariableNotSet(var) => var.clone(),
            LivenetError::SecrectKeyLoadError(_) => ENV_SECRET_KEY_PATH.to_string(),
            other => fail(&format!("Livenet misconfigured: {other:#}"))
        };

        // Without a TTY there is nobody to prompt — keep the fail-fast behaviour.
        if !io::stdin().is_terminal() {
            report_problem(&err);
            fail("No interactive terminal to prompt on. Set the variable(s) above and re-run.");
        }

        if !intro_shown {
            prettycli::warn("Livenet configuration is incomplete.");
            prettycli::info("Enter the values below to continue (empty input or Ctrl-D aborts).");
            prettycli::link(DOCS_URL);
            intro_shown = true;
        }

        // Explain a bad secret key before re-asking for its path.
        if let LivenetError::SecrectKeyLoadError(e) = &err {
            prettycli::error(&format!("Could not load the secret key: {e}"));
        }

        let (label, hint) = describe(&var);
        prettycli::info(&format!("{label}  (${var})"));
        if !hint.is_empty() {
            prettycli::info(&format!("  {hint}"));
        }

        match prompt() {
            Some(value) => {
                std::env::set_var(&var, &value);
                // Re-asking for the same variable (e.g. a bad key path) overwrites the old answer.
                match entered.iter_mut().find(|(v, _)| v == &var) {
                    Some(slot) => slot.1 = value,
                    None => entered.push((var, value))
                }
            }
            None => fail("Aborted: livenet configuration was not provided.")
        }
    }
}

/// Reads a single trimmed line from stdin. Returns `None` on EOF, error or empty input.
fn prompt() -> Option<String> {
    print!("  > ");
    if io::stdout().flush().is_err() {
        return None;
    }
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) | Err(_) => None,
        Ok(_) => {
            let value = input.trim().to_string();
            (!value.is_empty()).then_some(value)
        }
    }
}

/// Human-readable label and an example/hint for a known configuration variable.
fn describe(var: &str) -> (&'static str, &'static str) {
    match var {
        ENV_NODE_ADDRESS => ("Node RPC address", "e.g. http://localhost:11101"),
        ENV_CHAIN_NAME => ("Chain name", "e.g. casper-net-1"),
        ENV_EVENTS_URL => ("Events URL", "e.g. http://localhost:18101/events"),
        ENV_SECRET_KEY_PATH => ("Secret key path", "path to a PEM-encoded secret key"),
        _ => ("Configuration value", "")
    }
}

/// Prints the configuration problem and a link to the docs (used on the non-interactive path).
fn report_problem(err: &LivenetError) {
    match err {
        LivenetError::EnvVariableNotSet(var) => prettycli::error(&format!(
            "Livenet env misconfigured! {var} env var is missing."
        )),
        LivenetError::SecrectKeyLoadError(e) => {
            prettycli::error(&format!("Could not load the livenet secret key: {e}"))
        }
        other => prettycli::error(&format!("Livenet misconfigured: {other:#}"))
    }
    prettycli::info("Visit the official docs to read more:");
    prettycli::link(DOCS_URL);
}

/// Offers to persist the interactively-entered values to a `.env` file in the current directory.
///
/// Does nothing when nothing was entered (config was already complete) or the user declines.
fn offer_to_save(entered: &[(String, String)]) {
    if entered.is_empty() {
        return;
    }

    let path = match env_file_path() {
        Some(path) => path,
        None => return
    };

    let question = if path.exists() {
        format!(
            "Save this configuration to {}? (existing values are updated)",
            path.display()
        )
    } else {
        format!(
            "Save this configuration to {} for next time?",
            path.display()
        )
    };
    if !confirm(&question) {
        return;
    }

    match write_env_file(&path, entered) {
        Ok(()) => prettycli::info(&format!("Saved configuration to {}", path.display())),
        Err(e) => prettycli::warn(&format!("Could not write {}: {e}", path.display()))
    }
}

/// Path to the `.env` file in the current working directory, if it can be resolved.
fn env_file_path() -> Option<PathBuf> {
    std::env::current_dir().ok().map(|dir| dir.join(".env"))
}

/// Writes the entries to `path`, replacing any existing `VAR=...` line in place and appending the
/// rest. Other lines (comments, unrelated variables) are preserved verbatim.
fn write_env_file(path: &Path, entries: &[(String, String)]) -> io::Result<()> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut lines: Vec<String> = existing.lines().map(str::to_string).collect();

    for (var, value) in entries {
        let prefix = format!("{var}=");
        let new_line = format!("{var}={value}");
        match lines
            .iter_mut()
            .find(|l| l.trim_start().starts_with(&prefix))
        {
            Some(slot) => *slot = new_line,
            None => lines.push(new_line)
        }
    }

    let mut content = lines.join("\n");
    content.push('\n');
    std::fs::write(path, content)
}

/// Asks a yes/no question on the TTY. Defaults to `no` on empty input, EOF or error.
fn confirm(question: &str) -> bool {
    prettycli::info(question);
    print!("  [y/N] > ");
    if io::stdout().flush().is_err() {
        return false;
    }
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) | Err(_) => false,
        Ok(_) => matches!(input.trim().to_ascii_lowercase().as_str(), "y" | "yes")
    }
}

/// Prints an error and exits the process with a non-zero status.
fn fail(msg: &str) -> ! {
    prettycli::error(msg);
    std::process::exit(1);
}
