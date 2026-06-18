use crate::{
    cmd::WHOAMI_SUBCOMMAND, custom_types::CustomTypeSet, parser::motes_to_cspr,
    DeployedContractsContainer
};
use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::host::HostEnv;

use super::OdraCommand;

/// WhoamiCmd is a struct that represents the whoami command in the Odra CLI.
pub(crate) struct WhoamiCmd;

impl WhoamiCmd {
    /// Creates a new instance of `WhoamiCmd`.
    pub fn new() -> Self {
        Self
    }
}

impl OdraCommand for WhoamiCmd {
    fn run(
        &self,
        env: &HostEnv,
        _args: &ArgMatches,
        _types: &CustomTypeSet,
        _container: &DeployedContractsContainer
    ) -> Result<()> {
        let caller = env.caller();
        prettycli::info(&format!("Address: {}", caller.to_string()));
        let pk = env.public_key(&caller);
        prettycli::info(&format!("Public key: {pk}"));
        let balance_motes = env.balance_of(&caller);
        prettycli::info(&format!(
            "Balance: {} CSPR ({balance_motes} motes)",
            motes_to_cspr(balance_motes)
        ));
        Ok(())
    }
}

impl From<&WhoamiCmd> for Command {
    fn from(_value: &WhoamiCmd) -> Self {
        Command::new(WHOAMI_SUBCOMMAND).about("Prints the address of the current caller.")
    }
}
