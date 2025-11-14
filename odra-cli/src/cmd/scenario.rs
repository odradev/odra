use std::any::Any;

use crate::cmd::args::CommandArg;
use crate::cmd::SCENARIOS_SUBCOMMAND;
use crate::custom_types::CustomTypeSet;
use crate::{container::ContractError, types, DeployedContractsContainer};
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
        container: &mut DeployedContractsContainer,
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
        container: &mut DeployedContractsContainer
    ) -> Result<()> {
        args.subcommand()
            .map(|(scenario_name, scenario_args)| {
                self.scenarios
                    .iter()
                    .find(|cmd| cmd.name == scenario_name)
                    .map(|scenario| scenario.run(env, scenario_args, types, container))
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
/// The scenario command runs a [Scenario]. A scenario is a user-defined set of
/// actions that can be run in the Odra CLI.
struct ScenarioCmd {
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
        container: &mut DeployedContractsContainer
    ) -> Result<()> {
        let args = ScenarioArgs::new(args);
        env.set_captures_events(false);
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
    TypesError(#[from] types::Error),
    #[error("Missing scenario argument: {0}")]
    MissingScenarioArg(String)
}

impl From<OdraError> for ScenarioError {
    fn from(err: OdraError) -> Self {
        ScenarioError::OdraError {
            message: format!("{:?}", err)
        }
    }
}

/// ScenarioArgs is a struct that represents the arguments passed to a scenario.
pub struct ScenarioArgs<'a>(&'a ArgMatches);

impl<'a> ScenarioArgs<'a> {
    pub(crate) fn new(matches: &'a ArgMatches) -> Self {
        Self(matches)
    }

    pub fn get_single<T: NamedCLTyped + FromBytes + CLTyped>(
        &self,
        name: &str
    ) -> Result<T, ScenarioError> {
        let arg = self
            .0
            .try_get_one::<CLValue>(name)
            .map_err(|_| ScenarioError::ArgError(ArgError::UnexpectedArg(name.to_string())))?
            .ok_or(ArgError::MissingArg(name.to_string()))?;

        arg.clone()
            .into_t::<T>()
            .map_err(|_| ScenarioError::ArgError(ArgError::Deserialization))
    }

    pub fn get_many<T: NamedCLTyped + FromBytes + CLTyped>(
        &self,
        name: &str
    ) -> Result<Vec<T>, ScenarioError> {
        self.0
            .try_get_many::<CLValue>(name)
            .map_err(|_| ScenarioError::ArgError(ArgError::UnexpectedArg(name.to_string())))?
            .unwrap_or_default()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|value| value.clone().into_t::<T>())
            .collect::<Result<Vec<T>, _>>()
            .map_err(|_| ScenarioError::ArgError(ArgError::Deserialization))
    }
}

/// ArgError is an enum representing the different errors that can occur when parsing scenario arguments.
#[derive(Debug, Error, PartialEq)]
pub enum ArgError {
    #[error("Arg deserialization failed")]
    Deserialization,
    #[error("Multiple values expected")]
    ManyExpected,
    #[error("Single value expected")]
    SingleExpected,
    #[error("Missing arg: {0}")]
    MissingArg(String),
    #[error("Unexpected arg: {0}")]
    UnexpectedArg(String)
}

/// ScenarioMetadata is a trait that represents the metadata of a scenario.
pub trait ScenarioMetadata {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;
    use odra::schema::casper_contract_schema::NamedCLType;

    struct TestScenario;

    impl Scenario for TestScenario {
        fn args(&self) -> Vec<CommandArg> {
            vec![
                CommandArg::new("arg1", "First argument", NamedCLType::U32).required(),
                CommandArg::new("arg2", "Second argument", NamedCLType::String),
                CommandArg::new("arg3", "Optional argument", NamedCLType::String).list(),
            ]
        }

        fn run(
            &self,
            _env: &HostEnv,
            _container: &mut DeployedContractsContainer,
            args: ScenarioArgs
        ) -> core::result::Result<(), ScenarioError> {
            _ = args.get_single::<u32>("arg1")?;
            _ = args.get_single::<String>("arg2")?;
            _ = args.get_many::<String>("arg3")?;
            Ok(())
        }
    }

    impl ScenarioMetadata for TestScenario {
        const NAME: &'static str = "test_scenario";
        const DESCRIPTION: &'static str = "A test scenario for unit testing";
    }

    #[test]
    fn test_scenarios_command() {
        let mut scenarios = ScenariosCmd::default();
        scenarios.add_scenario(TestScenario);

        let cmd: Command = (&scenarios).into();

        assert_eq!(cmd.get_name(), SCENARIOS_SUBCOMMAND);
        assert!(cmd
            .get_subcommands()
            .any(|c| c.get_name() == "test_scenario"));
    }

    #[test]
    fn test_match_required_args() {
        let cmd = ScenarioCmd::new(TestScenario);
        let cmd: Command = (&cmd).into();

        let matches = cmd.try_get_matches_from(vec!["odra-cli", "--arg1", "1"]);
        assert!(matches.is_ok());
        assert_eq!(
            matches.unwrap().get_one::<CLValue>("arg1"),
            Some(&CLValue::from_t(1u32).unwrap())
        );
    }

    #[test]
    fn test_match_required_arg_missing() {
        let cmd = ScenarioCmd::new(TestScenario);
        let cmd: Command = (&cmd).into();

        let matches = cmd.try_get_matches_from(vec!["odra-cli"]);
        assert!(matches.is_err());
        assert_eq!(
            matches.unwrap_err().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn test_match_required_arg_with_wrong_type() {
        let cmd = ScenarioCmd::new(TestScenario);
        let cmd: Command = (&cmd).into();

        let matches = cmd.try_get_matches_from(vec!["odra-cli", "--arg1", "not_a_number"]);
        assert!(matches.is_err());
        assert_eq!(
            matches.unwrap_err().kind(),
            clap::error::ErrorKind::InvalidValue
        );
    }

    #[test]
    fn test_match_optional_args() {
        let cmd = ScenarioCmd::new(TestScenario);
        let cmd: Command = (&cmd).into();

        let matches = cmd.try_get_matches_from(vec![
            "odra-cli", "--arg1", "1", "--arg2", "test", "--arg3", "value1", "--arg3", "value2",
        ]);
        assert!(matches.is_ok());
        let matches = matches.unwrap();
        assert_eq!(
            matches.get_one::<CLValue>("arg1"),
            Some(&CLValue::from_t(1u32).unwrap())
        );
        assert_eq!(
            matches.get_one::<CLValue>("arg2"),
            Some(&CLValue::from_t("test".to_string()).unwrap())
        );
        assert_eq!(
            matches
                .get_many::<CLValue>("arg3")
                .unwrap()
                .collect::<Vec<_>>(),
            vec![
                &CLValue::from_t("value1".to_string()).unwrap(),
                &CLValue::from_t("value2".to_string()).unwrap()
            ]
        );
    }

    #[test]
    fn test_matching_list_args() {
        let scenario = ScenarioCmd::new(TestScenario);
        let cmd: Command = (&scenario).into();

        let matches = cmd.try_get_matches_from(vec![
            "odra-cli", "--arg1", "1", "--arg3", "value1", "--arg3", "value2",
        ]);
        assert!(matches.is_ok());
        let matches = matches.unwrap();
        assert_eq!(
            matches
                .get_many::<CLValue>("arg3")
                .unwrap()
                .collect::<Vec<_>>(),
            vec![
                &CLValue::from_t("value1".to_string()).unwrap(),
                &CLValue::from_t("value2".to_string()).unwrap()
            ]
        );

        // If we pass `arg2` multiple times, it should return an error
        let cmd: Command = (&scenario).into();
        let matches = cmd.try_get_matches_from(vec![
            "odra-cli", "--arg1", "1", "--arg2", "value1", "--arg2", "value2",
        ]);

        assert!(matches.is_err());
        assert_eq!(
            matches.unwrap_err().kind(),
            clap::error::ErrorKind::ArgumentConflict
        );
    }

    #[test]
    fn test_scenario_args_get_single() {
        let scenario = ScenarioCmd::new(TestScenario);
        let cmd: Command = (&scenario).into();

        let matches = cmd
            .try_get_matches_from(vec!["odra-cli", "--arg1", "42", "--arg2", "test_value"])
            .unwrap();

        let args = ScenarioArgs::new(&matches);
        let arg1: u32 = args.get_single("arg1").unwrap();
        assert_eq!(arg1, 42);

        let arg2: String = args.get_single("arg2").unwrap();
        assert_eq!(arg2, "test_value");
    }

    #[test]
    fn test_scenario_args_get_many() {
        let scenario = ScenarioCmd::new(TestScenario);
        let cmd: Command = (&scenario).into();

        let matches = cmd
            .try_get_matches_from(vec![
                "odra-cli", "--arg1", "42", "--arg3", "value1", "--arg3", "value2",
            ])
            .unwrap();

        let args = ScenarioArgs::new(&matches);
        let arg3: Vec<String> = args.get_many("arg3").unwrap();
        assert_eq!(arg3, vec!["value1".to_string(), "value2".to_string()]);
    }

    #[test]
    fn test_run_cmd() {
        let scenario = ScenarioCmd::new(TestScenario);
        let cmd: Command = (&scenario).into();

        let matches = cmd
            .try_get_matches_from(vec![
                "odra-cli",
                "--arg1",
                "42",
                "--arg2",
                "test_value",
                "--arg3",
                "value1",
                "--arg3",
                "value2",
            ])
            .unwrap();

        let env = test_utils::mock_host_env();
        let mut container = test_utils::mock_contracts_container();
        let result = scenario.run(&env, &matches, &CustomTypeSet::default(), &mut container);
        assert!(result.is_ok());
    }

    #[test]
    fn test_scenario_args_missing_arg() {
        let scenario = ScenarioCmd::new(TestScenario);
        let cmd: Command = (&scenario).into();

        let matches = cmd.get_matches_from(vec!["odra-cli", "--arg1", "1", "--arg2", "value1"]);
        ScenarioArgs(&matches)
            .get_single::<u32>("arg3")
            .expect_err("Expected an error for missing arg3");

        assert_eq!(ScenarioArgs(&matches).get_single::<u32>("arg1").unwrap(), 1);
    }
}
