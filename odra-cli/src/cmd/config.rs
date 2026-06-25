use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::host::HostEnv;
use serde_derive::Serialize;

use crate::{
    cmd::{CmdOutput, CONFIG_SUBCOMMAND},
    custom_types::CustomTypeSet,
    log,
    utils::get_default_contracts_file,
    DeployedContractsContainer
};

use super::OdraCommand;

const ENV_NODE_ADDRESS: &str = "ODRA_CASPER_LIVENET_NODE_ADDRESS";
const ENV_CHAIN_NAME: &str = "ODRA_CASPER_LIVENET_CHAIN_NAME";
const ENV_EVENTS_URL: &str = "ODRA_CASPER_LIVENET_EVENTS_URL";
const ENV_SECRET_KEY_PATH: &str = "ODRA_CASPER_LIVENET_SECRET_KEY_PATH";

/// Prints the resolved livenet configuration so a user can verify which network and account the
/// CLI is bound to before sending anything. Only the secret key *path* is shown, never its content.
pub(crate) struct ConfigCmd;

/// The resolved livenet configuration. Each `Option` is `None` when the backing env var is unset or
/// empty. Only the secret key *path* is ever included — never its contents.
#[derive(Serialize)]
pub(crate) struct ConfigReport {
    node_address: Option<String>,
    chain_name: Option<String>,
    events_url: Option<String>,
    secret_key_path: Option<String>,
    caller_address: String,
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
        log(format!("Caller address:  {}", self.caller_address));
        log(format!("Contracts file:  {}", self.contracts_file));
    }
}

impl OdraCommand for ConfigCmd {
    type Output = ConfigReport;

    fn exec(
        &self,
        env: &HostEnv,
        _args: &ArgMatches,
        _types: &CustomTypeSet,
        _container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        Ok(ConfigReport {
            node_address: env_var(ENV_NODE_ADDRESS),
            chain_name: env_var(ENV_CHAIN_NAME),
            events_url: env_var(ENV_EVENTS_URL),
            secret_key_path: env_var(ENV_SECRET_KEY_PATH),
            caller_address: env.caller().to_string(),
            contracts_file: get_default_contracts_file()
        })
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
}
