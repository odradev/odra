use std::path::PathBuf;
use std::{any::Any, collections::HashMap};

use crate::cmd::args::CommandArg;
use crate::SCENARIOS_SUBCOMMAND;
use crate::{container::ContractError, types, CustomTypeSet, DeployedContractsContainer};
use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::casper_types::{CLTyped, CLValue};
use odra::schema::NamedCLTyped;
use odra::{casper_types::bytesrepr::FromBytes, host::HostEnv, prelude::OdraError};
use thiserror::Error;

use super::OdraCommand;

/// Scenario is a trait that represents a custom scenario.
///
/// A scenario is a user-defined set of actions that can be run in the Odra CLI.
/// If you want to run a custom scenario that calls multiple entry points,
/// you need to implement this trait.
pub trait Scenario: Any {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }
    fn run(
        &self,
        env: &HostEnv,
        container: DeployedContractsContainer,
        args: ScenarioArgs
    ) -> core::result::Result<(), ScenarioError>;
}

#[derive(Default)]
pub(crate) struct ScenariosCmd {
    scenarios: Vec<ScenarioCmd>
}

impl ScenariosCmd {
    pub fn add_scenario<S: ScenarioMetadata + Scenario>(&mut self, scenario: S) {
        self.scenarios.push(ScenarioCmd::new(scenario));
    }
}

impl OdraCommand for ScenariosCmd {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        contracts_path: Option<PathBuf>
    ) -> Result<()> {
        args.subcommand()
            .map(|(scenario_name, scenario_args)| {
                self.scenarios
                    .iter()
                    .find(|cmd| cmd.name == scenario_name)
                    .map(|scenario| scenario.run(env, scenario_args, types, contracts_path))
                    .unwrap_or(Err(anyhow::anyhow!("No scenario found")))
            })
            .unwrap_or(Err(anyhow::anyhow!("No scenario found")))
    }
}

impl From<&ScenariosCmd> for Command {
    fn from(value: &ScenariosCmd) -> Self {
        Command::new(SCENARIOS_SUBCOMMAND)
            .about("Commands for interacting with scenarios")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommands(&value.scenarios)
    }
}

/// ScenarioCmd is a struct that represents a scenario command in the Odra CLI.
///
/// The scenario command runs a [Scenario]. A scenario is a user-defined set of actions that can be run in the Odra CLI.
pub(crate) struct ScenarioCmd {
    name: String,
    description: String,
    scenario: Box<dyn Scenario>
}

impl ScenarioCmd {
    pub fn new<S: ScenarioMetadata + Scenario>(scenario: S) -> Self {
        ScenarioCmd {
            name: S::NAME.to_string(),
            description: S::DESCRIPTION.to_string(),
            scenario: Box::new(scenario)
        }
    }
}

impl OdraCommand for ScenarioCmd {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        _types: &CustomTypeSet,
        contracts_path: Option<PathBuf>
    ) -> Result<()> {
        let container = DeployedContractsContainer::load(contracts_path)?;
        let args = ScenarioArgs::new(self.scenario.args(), args);

        self.scenario.run(env, container, args)?;
        Ok(())
    }
}

impl From<&ScenarioCmd> for Command {
    fn from(value: &ScenarioCmd) -> Self {
        Command::new(&value.name)
            .about(&value.description)
            .args(value.scenario.args())
    }
}

/// ScenarioError is an enum representing the different errors that can occur when running a scenario.
#[derive(Debug, Error)]
pub enum ScenarioError {
    #[error("Odra error: {message}")]
    OdraError { message: String },
    #[error("Contract read error: {0}")]
    ContractReadError(#[from] ContractError),
    #[error("Arg error")]
    ArgError(#[from] ArgError),
    #[error("Types error")]
    TypesError(#[from] types::Error)
}

impl From<OdraError> for ScenarioError {
    fn from(err: OdraError) -> Self {
        ScenarioError::OdraError {
            message: format!("{:?}", err)
        }
    }
}

/// ScenarioArgs is a struct that represents the arguments passed to a scenario.
pub struct ScenarioArgs(HashMap<String, ScenarioArg>);

impl ScenarioArgs {
    pub(crate) fn new(args: Vec<CommandArg>, matches: &ArgMatches) -> Self {
        let map = args
            .into_iter()
            .filter_map(|arg| {
                let arg_name = arg.name.clone();
                let values = matches
                    .get_many::<CLValue>(&arg_name)
                    .unwrap_or_default()
                    .map(|v| v.clone())
                    .collect::<Vec<_>>();

                if arg.required && values.is_empty() {
                    panic!("Missing argument: {}", arg.name);
                }

                let scenario_arg = match arg.is_list_element {
                    true => ScenarioArg::Many(values),
                    false => ScenarioArg::Single(values[0].clone())
                };

                Some((arg_name, scenario_arg))
            })
            .collect();
        Self(map)
    }

    pub fn get_single<T: NamedCLTyped + FromBytes + CLTyped>(
        &self,
        name: &str
    ) -> Result<T, ScenarioError> {
        let arg = self
            .0
            .get(name)
            .ok_or(ArgError::MissingArg(name.to_string()))?;

        let result = match arg {
            ScenarioArg::Single(value) => value
                .clone()
                .into_t::<T>()
                .map_err(|_| ArgError::Deserialization),
            ScenarioArg::Many(_) => Err(ArgError::SingleExpected)
        }?;
        Ok(result)
    }

    pub fn get_many<T: NamedCLTyped + FromBytes + CLTyped>(
        &self,
        name: &str
    ) -> Result<Vec<T>, ScenarioError> {
        let arg = self
            .0
            .get(name)
            .ok_or(ArgError::MissingArg(name.to_string()))?;
        match arg {
            ScenarioArg::Many(values) => values
                .iter()
                .map(|value| value.clone().into_t::<T>())
                .collect::<Result<Vec<T>, _>>()
                .map_err(|_| ScenarioError::ArgError(ArgError::Deserialization)),
            ScenarioArg::Single(_) => Err(ScenarioError::ArgError(ArgError::ManyExpected))
        }
    }
}

/// ArgError is an enum representing the different errors that can occur when parsing scenario arguments.
#[derive(Debug, Error)]
pub enum ArgError {
    #[error("Arg deserialization failed")]
    Deserialization,
    #[error("Multiple values expected")]
    ManyExpected,
    #[error("Single value expected")]
    SingleExpected,
    #[error("Missing arg: {0}")]
    MissingArg(String)
}

enum ScenarioArg {
    Single(CLValue),
    Many(Vec<CLValue>)
}

/// ScenarioMetadata is a trait that represents the metadata of a scenario.
pub trait ScenarioMetadata {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
}
