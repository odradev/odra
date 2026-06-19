use crate::{
    cmd::{CmdOutput, WHOAMI_SUBCOMMAND},
    custom_types::CustomTypeSet,
    log,
    parser::motes_to_cspr,
    DeployedContractsContainer
};
use anyhow::{Ok, Result};
use clap::{ArgMatches, Command};
use odra::host::HostEnv;
use serde_derive::Serialize;

use super::OdraCommand;

/// WhoamiCmd is a struct that represents the whoami command in the Odra CLI.
pub(crate) struct WhoamiCmd;

impl WhoamiCmd {
    /// Creates a new instance of `WhoamiCmd`.
    pub fn new() -> Self {
        Self
    }
}

/// The caller identity and balance. U512 amounts are serialized as strings to stay precise for
/// JSON consumers (the values can exceed JSON's safe integer range).
#[derive(Serialize)]
pub(crate) struct WhoamiReport {
    address: String,
    public_key: String,
    balance_motes: String,
    balance_cspr: String
}

impl CmdOutput for WhoamiReport {
    fn pretty_print(&self) {
        log(format!("Address: {}", self.address));
        log(format!("Public key: {}", self.public_key));
        log(format!(
            "Balance: {} CSPR ({} motes)",
            self.balance_cspr, self.balance_motes
        ));
    }
}

impl OdraCommand for WhoamiCmd {
    type Output = WhoamiReport;

    fn exec(
        &self,
        env: &HostEnv,
        _args: &ArgMatches,
        _types: &CustomTypeSet,
        _container: &DeployedContractsContainer
    ) -> Result<WhoamiReport> {
        let caller = env.caller();
        let balance_motes = env.balance_of(&caller);

        Ok(WhoamiReport {
            address: caller.to_string(),
            public_key: env.public_key(&caller).to_hex_string(),
            balance_cspr: motes_to_cspr(balance_motes),
            balance_motes: balance_motes.to_string()
        })
    }
}

impl From<&WhoamiCmd> for Command {
    fn from(_value: &WhoamiCmd) -> Self {
        Command::new(WHOAMI_SUBCOMMAND).about("Prints the address of the current caller.")
    }
}
