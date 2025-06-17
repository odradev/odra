use crate::{
    cmd::DEPLOY_SUBCOMMAND, container::ContractError, custom_types::CustomTypeSet,
    DeployedContractsContainer
};
use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::{host::HostEnv, prelude::OdraError};
use thiserror::Error;

use super::MutableCommand;

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

impl MutableCommand for DeployCmd {
    fn run(
        &self,
        env: &HostEnv,
        _args: &ArgMatches,
        _types: &CustomTypeSet,
        container: &mut DeployedContractsContainer
    ) -> Result<()> {
        self.script.deploy(env, container)?;
        Ok(())
    }
}

impl From<&DeployCmd> for Command {
    fn from(_value: &DeployCmd) -> Self {
        Command::new(DEPLOY_SUBCOMMAND).about("Runs the deploy script")
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

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;
    use odra::host::HostEnv;

    struct MockDeployScript;

    impl DeployScript for MockDeployScript {
        fn deploy(
            &self,
            _env: &HostEnv,
            _container: &mut DeployedContractsContainer
        ) -> core::result::Result<(), DeployError> {
            Ok(())
        }
    }

    #[test]
    fn deploy_cmd_run() {
        // This is a placeholder test to ensure the DeployCmd can be instantiated and run.
        let env = test_utils::mock_host_env();
        let cmd = DeployCmd::new(MockDeployScript);
        let mut container = test_utils::mock_contracts_container();
        let result = cmd.run(
            &env,
            &ArgMatches::default(),
            &CustomTypeSet::default(),
            &mut container
        );
        assert!(result.is_ok());
    }

    #[test]
    fn parsing_deploy_cmd() {
        // This is a placeholder test to ensure the DeployCmd can be converted to a Command.
        let cmd = DeployCmd::new(MockDeployScript);
        let command: Command = (&cmd).into();
        assert_eq!(command.get_name(), DEPLOY_SUBCOMMAND);
        assert!(command.get_about().is_some());
    }

    #[test]
    fn deploy_does_not_accept_args() {
        // This is a placeholder test to ensure the DeployCmd can be converted to a Command.
        let cmd = DeployCmd::new(MockDeployScript);
        let command: Command = (&cmd).into();

        let result = command.try_get_matches_from(vec!["test"]);
        assert!(result.is_ok());

        let command: Command = (&cmd).into();
        assert_eq!(command.get_arguments().count(), 0);
    }
}
