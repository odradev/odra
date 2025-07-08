use anyhow::Result;
use clap::ArgMatches;
use odra::host::HostEnv;

pub mod args;
mod contract;
mod deploy;
mod events;
mod main;
mod scenario;
mod whoami;

pub(crate) use contract::ContractsCmd;
pub(crate) use deploy::DeployCmd;
pub use deploy::{DeployError, DeployScript};
pub(crate) use events::PrintEventsCmd;
pub(crate) use main::MainCmd;
pub(crate) use scenario::ScenariosCmd;
pub use scenario::{Scenario, ScenarioArgs, ScenarioError, ScenarioMetadata};
pub(crate) use whoami::WhoamiCmd;

use crate::{custom_types::CustomTypeSet, DeployedContractsContainer};

pub(crate) const CONTRACTS_SUBCOMMAND: &str = "contract";
pub(crate) const SCENARIOS_SUBCOMMAND: &str = "scenario";
pub(crate) const DEPLOY_SUBCOMMAND: &str = "deploy";
pub(crate) const PRINT_EVENTS_SUBCOMMAND: &str = "print-events";
pub(crate) const WHOAMI_SUBCOMMAND: &str = "whoami";

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
