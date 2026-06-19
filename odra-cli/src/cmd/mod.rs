use anyhow::{Ok, Result};
use clap::ArgMatches;
use odra::host::HostEnv;

pub mod args;
mod completions;
mod config;
mod contract;
mod deploy;
mod events;
mod inspect;
mod main;
mod scenario;
mod status;
mod transfer;
mod whoami;

pub(crate) use completions::CompletionsCmd;
pub(crate) use config::ConfigCmd;
pub(crate) use contract::ContractsCmd;
pub(crate) use deploy::DeployCmd;
pub use deploy::{DeployError, DeployScript};
pub(crate) use events::PrintEventsCmd;
pub(crate) use inspect::InspectCmd;
pub(crate) use main::MainCmd;
pub(crate) use scenario::ScenariosCmd;
pub use scenario::{Scenario, ScenarioArgs, ScenarioError, ScenarioMetadata};
pub(crate) use status::StatusCmd;
pub(crate) use transfer::TransferCmd;
pub(crate) use whoami::WhoamiCmd;

use crate::{
    custom_types::CustomTypeSet,
    output::{print_json, OutputFormat},
    DeployedContractsContainer
};

pub(crate) const CONTRACTS_SUBCOMMAND: &str = "contract";
pub(crate) const SCENARIOS_SUBCOMMAND: &str = "scenario";
pub(crate) const DEPLOY_SUBCOMMAND: &str = "deploy";
pub(crate) const PRINT_EVENTS_SUBCOMMAND: &str = "print-events";
pub(crate) const WHOAMI_SUBCOMMAND: &str = "whoami";
pub(crate) const REPL_SUBCOMMAND: &str = "repl";
pub(crate) const STATUS_SUBCOMMAND: &str = "status";
pub(crate) const INSPECT_SUBCOMMAND: &str = "inspect";
pub(crate) const CONFIG_SUBCOMMAND: &str = "config";
pub(crate) const TRANSFER_SUBCOMMAND: &str = "transfer";
pub(crate) const COMPLETIONS_SUBCOMMAND: &str = "completions";

/// OdraCommand is a trait that represents a command that can be run in the Odra CLI.
pub(crate) trait OdraCommand {
    type Output: CmdOutput;

    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<()> {
        let result = self.exec(env, args, types, container)?;
        result.print(OutputFormat::from_args(args))?;
        Ok(())
    }

    fn exec(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<Self::Output>;
}

pub(crate) trait MutableCommand {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &mut DeployedContractsContainer
    ) -> Result<()>;
}

pub trait CmdOutput: serde::Serialize + Sized {
    fn pretty_print(&self);

    fn print(&self, format: OutputFormat) -> Result<()> {
        match format {
            OutputFormat::Json => print_json(self),
            OutputFormat::Human => {
                self.pretty_print();
                Ok(())
            }
        }
    }
}

impl CmdOutput for () {
    fn pretty_print(&self) {}
}
