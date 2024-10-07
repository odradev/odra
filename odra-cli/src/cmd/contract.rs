use anyhow::Result;
use clap::ArgMatches;
use odra_core::host::HostEnv; 
use odra_schema::casper_contract_schema::Entrypoint;

use crate::{entry_point, CustomTypeSet};

use super::OdraCommand;

/// ContractCmd is a struct that represents a contract command in the Odra CLI.
///
/// The contract command runs a contract with a given entry point.
pub(crate) struct ContractCmd {
    name: String,
    commands: Vec<Box<dyn OdraCommand>>,
}

impl ContractCmd {
    pub fn new<T: odra_schema::SchemaEntrypoints>(contract_name: String) -> Self {
        let commands = T::schema_entrypoints()
            .into_iter()
            .map(|entry_point| {
                Box::new(CallCmd {
                    contract_name: contract_name.clone(),
                    entry_point,
                }) as Box<dyn OdraCommand>
            })
            .collect::<Vec<_>>();
        ContractCmd {
            name: contract_name,
            commands,
        }
    }
}

impl OdraCommand for ContractCmd {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self, env: &HostEnv, args: &ArgMatches, types: &CustomTypeSet) -> Result<()> {
        args.subcommand()
            .map(|(entrypoint_name, entrypoint_args)| {
                self.commands
                    .iter()
                    .find(|cmd| cmd.name() == entrypoint_name)
                    .map(|entry_point| entry_point.run(env, entrypoint_args, types))
                    .unwrap_or(Err(anyhow::anyhow!("No entry point found")))
            })
            .unwrap_or(Err(anyhow::anyhow!("No entry point found")))
    }
}

/// CallCmd is a struct that represents a call command in the Odra CLI.
///
/// The call command runs a contract with a given entry point.
struct CallCmd {
    contract_name: String,
    entry_point: Entrypoint,
}

impl OdraCommand for CallCmd {
    fn name(&self) -> &str {
        &self.entry_point.name
    }

    fn run(&self, env: &HostEnv, args: &ArgMatches, types: &CustomTypeSet) -> Result<()> {
        let entry_point = &self.entry_point;
        let contract_name = &self.contract_name;

        let result = entry_point::call(env, contract_name, entry_point, args, types)?;
        prettycli::info(&result);
        Ok(())
    }
}
