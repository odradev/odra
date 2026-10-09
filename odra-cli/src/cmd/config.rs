use anyhow::Result;
use clap::{ArgMatches, Command};
use serde_derive::Serialize;

use crate::{
    cmd::{CmdOutput, CONFIG_SUBCOMMAND},
    log,
    output::OutputFormat,
    utils::get_default_contracts_file
};

const ENV_NODE_ADDRESS: &str = "ODRA_CASPER_LIVENET_NODE_ADDRESS";
const ENV_CHAIN_NAME: &str = "ODRA_CASPER_LIVENET_CHAIN_NAME";
const ENV_EVENTS_URL: &str = "ODRA_CASPER_LIVENET_EVENTS_URL";
const ENV_SECRET_KEY_PATH: &str = "ODRA_CASPER_LIVENET_SECRET_KEY_PATH";

/// Prints the resolved livenet configuration so a user can verify which network and account the
/// CLI is bound to before sending anything. Only the secret key *path* is shown, never its content.
///
/// It never prompts and needs no livenet connection: an incomplete configuration is reported (the
/// missing variables and why the caller can't be resolved) instead of aborting the command.
pub(crate) struct ConfigCmd;

/// The resolved livenet configuration. Each `Option` is `None` when the backing env var is unset or
/// empty. Only the secret key *path* is ever included — never its contents.
#[derive(Serialize)]
pub(crate) struct ConfigReport {
    node_address: Option<String>,
    chain_name: Option<String>,
    events_url: Option<String>,
    secret_key_path: Option<String>,
    /// `None` when the configuration is incomplete; `problem` then says why.
    caller_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    problem: Option<String>,
    contracts_file: String
}

impl CmdOutput for ConfigReport {
    fn pretty_print(&self) {
        log("Livenet configuration:");
        print_var("Node address", &self.node_address, ENV_NODE_ADDRESS);
        print_var("Chain name", &self.chain_name, ENV_CHAIN_NAME);
        print_var("Events URL", &self.events_url, ENV_EVENTS_URL);
        print_var(
            "Secret key path",
            &self.secret_key_path,
            ENV_SECRET_KEY_PATH
        );
        match &self.caller_address {
            Some(caller) => log(format!("Caller address:  {caller}")),
            None => prettycli::warn("Caller address:  <unavailable>")
        }
        log(format!("Contracts file:  {}", self.contracts_file));
        if let Some(problem) = &self.problem {
            prettycli::warn(problem);
        }
    }
}

impl ConfigCmd {
    /// Prints the configuration report.
    ///
    /// `caller` is the resolved caller address, or the reason the livenet environment couldn't be
    /// set up. The environment variables are read after resolving it, so values loaded from a
    /// `.env` file along the way are included.
    pub fn run(&self, caller: Result<String, String>, args: &ArgMatches) -> Result<()> {
        self.report(caller).print(OutputFormat::from_args(args))
    }

    fn report(&self, caller: Result<String, String>) -> ConfigReport {
        let (caller_address, problem) = match caller {
            Ok(caller) => (Some(caller), None),
            Err(problem) => (None, Some(problem))
        };
        ConfigReport {
            node_address: env_var(ENV_NODE_ADDRESS),
            chain_name: env_var(ENV_CHAIN_NAME),
            events_url: env_var(ENV_EVENTS_URL),
            secret_key_path: env_var(ENV_SECRET_KEY_PATH),
            caller_address,
            problem,
            contracts_file: get_default_contracts_file()
        }
    }
}

/// Reads an env var, mapping unset/empty to `None`.
fn env_var(var: &str) -> Option<String> {
    std::env::var(var).ok().filter(|v| !v.is_empty())
}

/// Prints `<label>: <value>`, or a warning when the variable is unset/empty.
fn print_var(label: &str, value: &Option<String>, var: &str) {
    match value {
        Some(value) => prettycli::info(&format!("  {label}: {value}")),
        None => prettycli::warn(&format!("  {label}: <not set> (${var})"))
    }
}

impl From<&ConfigCmd> for Command {
    fn from(_value: &ConfigCmd) -> Self {
        Command::new(CONFIG_SUBCOMMAND).about("Prints the resolved livenet configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_config_command() {
        let clap_cmd: Command = (&ConfigCmd).into();
        assert_eq!(clap_cmd.get_name(), CONFIG_SUBCOMMAND);
        assert!(clap_cmd.get_about().is_some());
    }

    #[test]
    fn reports_a_misconfiguration_instead_of_failing() {
        let report = ConfigCmd.report(Err("ODRA_X env var is missing.".to_string()));
        assert_eq!(report.caller_address, None);
        assert_eq!(
            report.problem.as_deref(),
            Some("ODRA_X env var is missing.")
        );

        let report = ConfigCmd.report(Ok("account-hash-00".to_string()));
        assert_eq!(report.caller_address.as_deref(), Some("account-hash-00"));
        assert_eq!(report.problem, None);
        let json = serde_json::to_value(&report).unwrap();
        assert!(json.get("problem").is_none());
    }
}
