use anyhow::Result;
use clap::ArgMatches;
use contract::ContractCmd;
use deploy::DeployCmd;
use odra::host::HostEnv;
use odra::schema::SchemaEntrypoints;
use scenario::ScenarioCmd;

use crate::{CustomTypeSet, DeployScript, Scenario, ScenarioMetadata};

pub mod contract;
pub mod deploy;
pub mod scenario;

/// OdraCommand is a trait that represents a command that can be run in the Odra CLI.
pub(crate) trait OdraCommand {
    fn name(&self) -> &str;
    fn run(&self, env: &HostEnv, args: &ArgMatches, types: &CustomTypeSet) -> Result<()>;
}

/// OdraCliCommand is an enum that represents the different commands that can be run in the Odra CLI.
pub(crate) enum OdraCliCommand {
    Deploy(DeployCmd),
    Scenario(ScenarioCmd),
    Contract(ContractCmd)
}

impl OdraCliCommand {
    pub fn new_deploy(script: impl DeployScript + 'static) -> Self {
        OdraCliCommand::Deploy(DeployCmd {
            script: Box::new(script)
        })
    }

    pub fn new_scenario<S: ScenarioMetadata + Scenario>(scenario: S) -> Self {
        OdraCliCommand::Scenario(ScenarioCmd::new(scenario))
    }

    pub fn new_contract<T: SchemaEntrypoints>(contract_name: String) -> Self {
        OdraCliCommand::Contract(ContractCmd::new::<T>(contract_name))
    }
}

impl OdraCommand for OdraCliCommand {
    fn name(&self) -> &str {
        match self {
            OdraCliCommand::Deploy(deploy) => deploy.name(),
            OdraCliCommand::Scenario(scenario) => scenario.name(),
            OdraCliCommand::Contract(contract) => contract.name()
        }
    }

    fn run(&self, env: &HostEnv, args: &ArgMatches, types: &CustomTypeSet) -> Result<()> {
        match self {
            OdraCliCommand::Deploy(deploy) => deploy.run(env, args, types),
            OdraCliCommand::Scenario(scenario) => scenario.run(env, args, types),
            OdraCliCommand::Contract(contract) => contract.run(env, args, types)
        }
    }
}
