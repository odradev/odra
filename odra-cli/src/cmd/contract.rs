use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::schema::casper_contract_schema::{CustomType, Entrypoint};
use odra::schema::{SchemaCustomTypes, SchemaEntrypoints, SchemaEvents};
use odra::OdraContract;
use odra::{contract_def::HasIdent, host::HostEnv};

use crate::cmd::args::Arg;
use crate::entry_point::CallError;
use crate::{args, entry_point, CustomTypeSet, CONTRACTS_SUBCOMMAND};

use super::OdraCommand;

#[derive(Default)]
pub(crate) struct ContractsCmd {
    contracts: Vec<ContractCmd>
}

impl ContractsCmd {
    pub fn add_contract<T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents>(
        &mut self
    ) {
        self.contracts.push(ContractCmd::new::<T>());
    }
}

impl OdraCommand for ContractsCmd {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        contracts_path: Option<PathBuf>
    ) -> Result<()> {
        args.subcommand()
            .map(|(contract_name, contract_args)| {
                self.contracts
                    .iter()
                    .find(|cmd| cmd.name == contract_name)
                    .map(|contract| contract.run(env, contract_args, types, contracts_path))
                    .unwrap_or(Err(anyhow::anyhow!("No contract found")))
            })
            .unwrap_or(Err(anyhow::anyhow!("No contract found")))
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
    name: String,
    entry_points: Vec<CallCmd>
}

impl ContractCmd {
    pub fn new<T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents>() -> Self {
        let contract_name = T::HostRef::ident();
        let entry_points = T::schema_entrypoints()
            .into_iter()
            .filter(|entry_point| entry_point.name != "init")
            .map(CallCmd::new::<T>)
            .collect::<Vec<_>>();
        ContractCmd {
            name: contract_name,
            entry_points
        }
    }
}

impl OdraCommand for ContractCmd {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        contracts_path: Option<PathBuf>
    ) -> Result<()> {
        args.subcommand()
            .map(|(entrypoint_name, entrypoint_args)| {
                self.entry_points
                    .iter()
                    .find(|cmd| cmd.entry_point.name == entrypoint_name)
                    .map(|entry_point| entry_point.run(env, entrypoint_args, types, contracts_path))
                    .unwrap_or(Err(CallError::EntryPointNotFound {
                        entry_point: entrypoint_name.to_string(),
                        contract_name: self.name.clone()
                    }
                    .into()))
            })
            .unwrap_or(Err(CallError::NoEntryPointFound {
                contract_name: self.name.clone()
            }
            .into()))
    }
}

impl From<&ContractCmd> for Command {
    fn from(value: &ContractCmd) -> Self {
        Command::new(&value.name)
            .about(format!(
                "Commands for interacting with the {} contract",
                &value.name
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
    contract_name: String,
    entry_point: Entrypoint,
    custom_types: BTreeSet<CustomType>
}

impl CallCmd {
    pub fn new<T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents>(
        entry_point: Entrypoint
    ) -> Self {
        let mut custom_types = BTreeSet::new();
        custom_types.extend(T::schema_types().into_iter().flatten());
        custom_types.extend(<T as SchemaEvents>::custom_types().into_iter().flatten());
        CallCmd {
            contract_name: T::HostRef::ident(),
            entry_point,
            custom_types
        }
    }
}

impl OdraCommand for CallCmd {
    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        contracts_path: Option<PathBuf>
    ) -> Result<()> {
        let entry_point = &self.entry_point;
        let contract_name = &self.contract_name;

        let result =
            entry_point::call(env, contract_name, entry_point, args, types, contracts_path)?;
        prettycli::info(&result);
        Ok(())
    }
}

impl From<&CallCmd> for Command {
    fn from(value: &CallCmd) -> Self {
        let mut cmd = Command::new(&value.entry_point.name)
            .about(value.entry_point.description.clone().unwrap_or_default())
            .args(args::entry_point_args(
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
