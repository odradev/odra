use std::collections::BTreeSet;

use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::schema::casper_contract_schema::{CustomType, Entrypoint};
use odra::schema::{SchemaCustomTypes, SchemaEntrypoints, SchemaEvents};
use odra::OdraContract;
use odra::{contract_def::HasIdent, host::HostEnv};

use serde_derive::Serialize;

use crate::cmd::args::Arg;
use crate::cmd::{CmdOutput, CONTRACTS_SUBCOMMAND};
use crate::custom_types::CustomTypeSet;
use crate::entry_point::{self, ContractEvents};
use crate::{log, DeployedContractsContainer};

use super::OdraCommand;

#[derive(Default)]
pub(crate) struct ContractsCmd {
    contracts: Vec<ContractCmd>
}

impl ContractsCmd {
    pub fn add_contract_named<
        T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents
    >(
        &mut self,
        package_name: String
    ) {
        self.contracts
            .push(ContractCmd::new_named::<T>(Some(package_name)));
    }

    pub fn add_contract<T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents>(
        &mut self
    ) {
        self.contracts.push(ContractCmd::new::<T>());
    }
}

impl OdraCommand for ContractsCmd {
    type Output = CallResultReport;

    fn exec(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        let (contract_name, contract_args) = args
            .subcommand()
            .ok_or_else(|| anyhow::anyhow!("No contract found"))?;
        let contract = self
            .contracts
            .iter()
            .find(|cmd| cmd.package_name == contract_name)
            .ok_or_else(|| anyhow::anyhow!("No contract found"))?;
        contract.exec(env, contract_args, types, container)
    }
}

impl From<&ContractsCmd> for Command {
    fn from(value: &ContractsCmd) -> Self {
        Command::new(CONTRACTS_SUBCOMMAND)
            .about("Commands for interacting with contracts")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommands(&value.contracts)
    }
}

/// ContractCmd is a struct that represents a contract command in the Odra CLI.
///
/// The contract command runs a contract with a given entry point.
struct ContractCmd {
    contract_name: String,
    package_name: String,
    entry_points: Vec<CallCmd>
}

impl ContractCmd {
    pub fn new<T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents>() -> Self {
        Self::new_named::<T>(None)
    }

    pub fn new_named<T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents>(
        package_name: Option<String>
    ) -> Self {
        let contract_name = T::HostRef::ident();
        let package_name = package_name.unwrap_or(T::HostRef::ident());
        let entry_points = T::schema_entrypoints()
            .into_iter()
            .filter(|entry_point| entry_point.name != "init")
            .map(|entry_point| CallCmd::new::<T>(entry_point, package_name.clone()))
            .collect::<Vec<_>>();
        ContractCmd {
            contract_name,
            package_name,
            entry_points
        }
    }
}

impl OdraCommand for ContractCmd {
    type Output = CallResultReport;

    fn exec(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        let (entrypoint_name, entrypoint_args) =
            args.subcommand()
                .ok_or_else(|| entry_point::CallError::NoEntryPointFound {
                    contract_name: self.package_name.clone()
                })?;
        let entry_point = self
            .entry_points
            .iter()
            .find(|cmd| cmd.entry_point.name == entrypoint_name)
            .ok_or_else(|| entry_point::CallError::EntryPointNotFound {
                entry_point: entrypoint_name.to_string(),
                contract_name: self.contract_name.clone()
            })?;
        entry_point.exec(env, entrypoint_args, types, container)
    }
}

impl From<&ContractCmd> for Command {
    fn from(value: &ContractCmd) -> Self {
        Command::new(&value.package_name)
            .about(format!(
                "Commands for interacting with the {} contract",
                &value.package_name
            ))
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommands(&value.entry_points)
    }
}

/// CallCmd is a struct that represents a call command in the Odra CLI.
///
/// The call command runs a contract with a given entry point.
struct CallCmd {
    package_name: String,
    entry_point: Entrypoint,
    custom_types: BTreeSet<CustomType>
}

impl CallCmd {
    pub fn new<T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents>(
        entry_point: Entrypoint,
        package_name: String
    ) -> Self {
        let mut custom_types = BTreeSet::new();
        custom_types.extend(T::schema_types().into_iter().flatten());
        custom_types.extend(<T as SchemaEvents>::custom_types().into_iter().flatten());
        CallCmd {
            package_name,
            entry_point,
            custom_types
        }
    }
}

impl OdraCommand for CallCmd {
    type Output = CallResultReport;

    #[cfg(not(test))]
    fn exec(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        let entry_point = &self.entry_point;
        let contract_name = &self.package_name;

        let outcome = entry_point::call(env, contract_name, entry_point, args, types, container)?;
        Ok(CallResultReport {
            contract: contract_name.clone(),
            entry_point: entry_point.name.clone(),
            result: Some(outcome.result).filter(|r| !r.is_empty()),
            events: outcome.events
        })
    }

    // In tests there is no live backend to call, so the entry point is not executed; we only
    // validate that the required arguments were supplied and return an empty report.
    #[cfg(test)]
    fn exec(
        &self,
        _env: &HostEnv,
        args: &ArgMatches,
        _types: &CustomTypeSet,
        _container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        for a in &self.entry_point.arguments {
            if !args.contains_id(&a.name) {
                return Err(entry_point::CallError::ExecutionError {
                    package_name: self.package_name.clone(),
                    method: self.entry_point.name.clone(),
                    message: format!("Missing required argument: {}", a.name)
                }
                .into());
            }
        }
        Ok(CallResultReport {
            contract: self.package_name.clone(),
            entry_point: self.entry_point.name.clone(),
            result: None,
            events: Vec::new()
        })
    }
}

/// The result of a contract call: the decoded return value (absent when the entry point returns
/// nothing) plus any events captured via `--print-events` on a mutable call.
#[derive(Serialize)]
pub(crate) struct CallResultReport {
    contract: String,
    entry_point: String,
    result: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    events: Vec<ContractEvents>
}

impl CmdOutput for CallResultReport {
    fn pretty_print(&self) {
        for group in &self.events {
            log(format!(
                "Captured {} events for contract '{}'",
                group.events.len(),
                group.contract
            ));
            for (i, event) in group.events.iter().enumerate() {
                log(format!("Event {}: {}", i + 1, event));
            }
        }
        match &self.result {
            Some(result) => log(format!("Call result: {result}")),
            None => log("Call executed successfully, but no result was returned.")
        }
    }
}

impl From<&CallCmd> for Command {
    fn from(value: &CallCmd) -> Self {
        let mut cmd = Command::new(&value.entry_point.name)
            .about(value.entry_point.description.clone().unwrap_or_default())
            .args(entry_point::cmd_args::entry_point_args(
                &value.entry_point,
                &value.custom_types
            ))
            .arg(Arg::AttachedValue);
        // If the entry point is mutable, a transaction is being sent, so we need to
        // provide the gas argument.
        if value.entry_point.is_mutable {
            cmd = cmd.arg(Arg::Gas).arg(Arg::PrintEvents);
        }
        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{self, TestContract};

    #[test]
    fn test_contracts_cmd() {
        let mut cmd = ContractsCmd::default();
        cmd.add_contract::<TestContract>();

        assert_eq!(cmd.contracts.len(), 1);
        assert_eq!(cmd.contracts[0].contract_name, "TestContract");

        let clap_cmd: Command = (&cmd).into();
        assert_eq!(clap_cmd.get_name(), CONTRACTS_SUBCOMMAND);
        assert!(clap_cmd
            .get_subcommands()
            .any(|c| c.get_name() == "TestContract"));
    }

    #[test]
    fn test_contract_cmd() {
        let cmd = ContractCmd::new::<TestContract>();

        assert_eq!(cmd.contract_name, "TestContract");
        assert_eq!(cmd.entry_points.len(), 6);
    }

    #[test]
    fn parsing_fails_if_entry_point_missing() {
        let cmd = ContractCmd::new::<TestContract>();

        let clap_cmd: Command = (&cmd).into();
        let result = clap_cmd.try_get_matches_from(vec!["test"]);
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
    }

    #[test]
    fn parsing_fails_if_invalid_entry_point() {
        let cmd = ContractCmd::new::<TestContract>();

        let clap_cmd: Command = (&cmd).into();
        let result = clap_cmd.try_get_matches_from(vec!["test", "sub"]);
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::InvalidSubcommand
        );
    }

    #[test]
    fn parsing_entry_point() {
        let cmd = ContractCmd::new::<TestContract>();

        let clap_cmd: Command = (&cmd).into();
        let result = clap_cmd.try_get_matches_from(vec!["test", "add", "--x", "5", "--y", "10"]);
        assert!(result.is_ok());
    }

    #[test]
    fn parsing_entry_point_fails_if_arg_is_missing() {
        let cmd = ContractCmd::new::<TestContract>();

        let clap_cmd: Command = (&cmd).into();
        let result = clap_cmd.try_get_matches_from(vec!["test", "add", "--x", "5"]);
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn parsing_entry_point_fails_if_wrong_arg() {
        let cmd = ContractCmd::new::<TestContract>();

        let clap_cmd: Command = (&cmd).into();
        let result = clap_cmd.try_get_matches_from(vec!["test", "add", "--x", "5", "--yy", "10"]);
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::UnknownArgument
        );
    }

    #[test]
    fn gas_required_if_mutable_entry_point() {
        let cmd = ContractCmd::new::<TestContract>();

        let clap_cmd: Command = (&cmd).into();
        let result = clap_cmd.try_get_matches_from(vec!["test", "mutable"]);
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );

        let clap_cmd: Command = (&cmd).into();
        let result =
            clap_cmd.try_get_matches_from(vec!["test", "mutable", "--gas", "10000000000000"]);
        assert!(result.is_ok());

        let clap_cmd: Command = (&cmd).into();
        // The minimum gas value is 2500000000, so this should fail.
        let result = clap_cmd.try_get_matches_from(vec!["test", "mutable", "--gas", "2400000000"]);
        assert_eq!(
            result.unwrap_err().kind(),
            clap::error::ErrorKind::ValueValidation
        );
    }

    #[test]
    fn attached_value_if_allowed() {
        let cmd = ContractCmd::new::<TestContract>();

        let clap_cmd: Command = (&cmd).into();
        let result = clap_cmd.try_get_matches_from(vec![
            "test",
            "mutable",
            "--gas",
            "10000000000000",
            "--attached_value",
            "1000",
        ]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run() {
        let cmd = ContractCmd::new::<TestContract>();
        let clap_cmd: Command = (&cmd).into();
        let args = clap_cmd.get_matches_from(vec!["test", "add", "--x", "5", "--y", "10"]);
        let env = test_utils::mock_host_env();
        let container = test_utils::mock_contracts_container();
        let result = cmd.run(&env, &args, &CustomTypeSet::new(), &container);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parsing_arguments() {
        let cmd = ContractCmd::new::<TestContract>();

        let clap_cmd: Command = (&cmd).into();
        let args = clap_cmd.try_get_matches_from(vec![
            "test",
            "various_args",
            "--a",
            "account-hash-5e3725bec4389ea63151903f5c9005233d19a569c5e593e5bbd83b05714f7364",
            "--b",
            "some:account-hash-5e3725bec4389ea63151903f5c9005233d19a569c5e593e5bbd83b05714f7364",
            "--c",
            "'alice','bob','caroline'",
            "--d",
            "ok:100",
            "--e",
            "err:Error message",
            "--f",
            "('single_value')",
            "--g",
            "('value1':'value2')",
            "--h",
            "('value1':'value2':'value3')",
            "--i",
            "key1=1,key2=2",
            "--j",
            "0x12,0x34,0x56,0x78,0x9a,0xbc,0xde,0xf0",
        ]);
        assert!(args.is_ok());

        // Enum: valid variant names are accepted
        for variant in &["Active", "Paused", "Terminated"] {
            let clap_cmd: Command = (&cmd).into();
            let result = clap_cmd.try_get_matches_from(vec![
                "test",
                "set_status",
                "--gas",
                "10000000000",
                "--status",
                variant,
            ]);
            assert!(
                result.is_ok(),
                "Expected '{}' to be a valid variant",
                variant
            );
        }

        // Enum: unknown variant name is rejected
        let clap_cmd: Command = (&cmd).into();
        let result = clap_cmd.try_get_matches_from(vec![
            "test",
            "set_status",
            "--gas",
            "10000000000",
            "--status",
            "Unknown",
        ]);
        assert!(result.is_err(), "Expected 'Unknown' to be rejected");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Active")
                && err_msg.contains("Paused")
                && err_msg.contains("Terminated"),
            "Error should list valid variants, got: {}",
            err_msg
        );
    }
}
