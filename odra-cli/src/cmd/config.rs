use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::host::HostEnv;

use crate::{
    cmd::CONFIG_SUBCOMMAND, custom_types::CustomTypeSet, utils::get_default_contracts_file,
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

impl OdraCommand for ConfigCmd {
    fn run(
        &self,
        env: &HostEnv,
        _args: &ArgMatches,
        _types: &CustomTypeSet,
        _container: &DeployedContractsContainer
    ) -> Result<()> {
        prettycli::info("Livenet configuration:");
        print_var("Node address", ENV_NODE_ADDRESS);
        print_var("Chain name", ENV_CHAIN_NAME);
        print_var("Events URL", ENV_EVENTS_URL);
        print_var("Secret key path", ENV_SECRET_KEY_PATH);
        prettycli::info(&format!("Caller address:  {}", env.caller().to_string()));
        prettycli::info(&format!(
            "Contracts file:  {}",
            get_default_contracts_file()
        ));
        Ok(())
    }
}

/// Prints `<label>: <value>`, or a warning when the variable is unset/empty.
fn print_var(label: &str, var: &str) {
    match std::env::var(var) {
        Ok(value) if !value.is_empty() => prettycli::info(&format!("  {label}: {value}")),
        _ => prettycli::warn(&format!("  {label}: <not set> (${var})"))
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
