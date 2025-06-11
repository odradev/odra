use std::path::PathBuf;

use crate::{
    container::ContractError, CustomTypeSet, DeployedContractsContainer, DEPLOY_SUBCOMMAND
};
use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::{host::HostEnv, prelude::OdraError};
use thiserror::Error;

use super::OdraCommand;

/// DeployCmd is a struct that represents the deploy command in the Odra CLI.
///
/// The deploy command runs the [DeployScript].
pub(crate) struct DeployCmd {
    pub script: Box<dyn DeployScript>
}

impl DeployCmd {
    /// Creates a new instance of `DeployCmd` with the provided script.
    pub fn new(script: impl DeployScript + 'static) -> Self {
        DeployCmd {
            script: Box::new(script)
        }
    }
}

impl OdraCommand for DeployCmd {
    fn run(
        &self,
        env: &HostEnv,
        _args: &ArgMatches,
        _types: &CustomTypeSet,
        contracts_path: Option<PathBuf>
    ) -> Result<()> {
        let mut container = DeployedContractsContainer::new(contracts_path)?;
        self.script.deploy(env, &mut container)?;
        Ok(())
    }
}

impl From<&DeployCmd> for Command {
    fn from(_value: &DeployCmd) -> Self {
        Command::new(DEPLOY_SUBCOMMAND)
            .about("Runs the deploy script")
            .arg_required_else_help(true)
    }
}

/// Script that deploys contracts to the blockchain and stores contract data for further use.
///
/// In a deploy script, you can define the contracts that you want to deploy to the blockchain
/// and write metadata to the container.
pub trait DeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer
    ) -> core::result::Result<(), DeployError>;
}

/// Error that occurs during contract deployment.
#[derive(Debug, Error)]
pub enum DeployError {
    #[error("Deploy error: {message}")]
    OdraError { message: String },
    #[error("Contract read error: {0}")]
    ContractReadError(#[from] ContractError)
}

impl From<OdraError> for DeployError {
    fn from(err: OdraError) -> Self {
        DeployError::OdraError {
            message: format!("{:?}", err)
        }
    }
}
