use std::{any::Any, collections::HashMap};

use crate::{
    args::CommandArg, container::ContractError, types, CustomTypeSet, DeployedContractsContainer
};
use anyhow::Result;
use clap::ArgMatches;
use odra::schema::NamedCLTyped;
use odra::{casper_types::bytesrepr::FromBytes, host::HostEnv, OdraError};
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

/// ScenarioCmd is a struct that represents a scenario command in the Odra CLI.
///
/// The scenario command runs a [Scenario]. A scenario is a user-defined set of actions that can be run in the Odra CLI.
pub(crate) struct ScenarioCmd {
    name: String,
    scenario: Box<dyn Scenario>
}

impl ScenarioCmd {
    pub fn new<S: ScenarioMetadata + Scenario>(scenario: S) -> Self {
        ScenarioCmd {
            name: S::NAME.to_string(),
            scenario: Box::new(scenario)
        }
    }
}

impl OdraCommand for ScenarioCmd {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, env: &HostEnv, args: &ArgMatches, _types: &CustomTypeSet) -> Result<()> {
        let container = DeployedContractsContainer::load()?;
        let args = ScenarioArgs::new(self.scenario.args(), args);

        self.scenario.run(env, container, args)?;
        Ok(())
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
                let arg_name = &arg.name;
                let values = matches.get_many::<String>(arg_name);

                if arg.required && values.is_none() {
                    panic!("Missing argument: {}", arg.name);
                }
                values.as_ref()?;
                let values = values
                    .expect("Arg not found")
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>();
                let scenario_arg = match arg.is_list_element {
                    true => ScenarioArg::Many(values),
                    false => ScenarioArg::Single(values[0].clone())
                };

                Some((arg_name.clone(), scenario_arg))
            })
            .collect();
        Self(map)
    }

    pub fn get_single<T: NamedCLTyped + FromBytes>(&self, name: &str) -> Result<T, ScenarioError> {
        let arg = self
            .0
            .get(name)
            .ok_or(ArgError::MissingArg(name.to_string()))?;

        let result = match arg {
            ScenarioArg::Single(value) => {
                let bytes = types::into_bytes(&T::ty(), value)?;
                T::from_bytes(&bytes)
                    .map_err(|_| ArgError::Deserialization)
                    .map(|t| t.0)
            }
            ScenarioArg::Many(_) => Err(ArgError::SingleExpected)
        }?;
        Ok(result)
    }

    pub fn get_many<T: NamedCLTyped + FromBytes>(
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
                .map(|value| {
                    let bytes = types::into_bytes(&T::ty(), value);
                    bytes.map_err(ScenarioError::TypesError).and_then(|bytes| {
                        T::from_bytes(&bytes)
                            .map_err(|_| ScenarioError::ArgError(ArgError::Deserialization))
                            .map(|t| t.0)
                    })
                })
                .collect::<Result<Vec<T>, ScenarioError>>(),
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
    Single(String),
    Many(Vec<String>)
}

/// ScenarioMetadata is a trait that represents the metadata of a scenario.
pub trait ScenarioMetadata {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
}
