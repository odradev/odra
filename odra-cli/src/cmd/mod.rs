use anyhow::Result;
use clap::ArgMatches;
use odra::host::HostEnv;

use crate::{CustomTypeSet, DeployedContractsContainer};

pub mod args;
mod contract;
mod deploy;
mod events;
mod main;
mod scenario;

pub(crate) use contract::ContractsCmd;
pub(crate) use deploy::DeployCmd;
pub use deploy::{DeployError, DeployScript};
pub(crate) use events::PrintEventsCmd;
pub(crate) use main::MainCmd;
pub(crate) use scenario::ScenariosCmd;
pub use scenario::{Scenario, ScenarioArgs, ScenarioError, ScenarioMetadata};

pub const CONTRACTS_SUBCOMMAND: &str = "contract";
pub const SCENARIOS_SUBCOMMAND: &str = "scenario";
pub const DEPLOY_SUBCOMMAND: &str = "deploy";
pub const PRINT_EVENTS_SUBCOMMAND: &str = "print-events";

/// OdraCommand is a trait that represents a command that can be run in the Odra CLI.
pub(crate) trait OdraCommand {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<()>;
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
