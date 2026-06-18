use std::collections::BTreeSet;

use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::schema::casper_contract_schema::{CustomType, Entrypoint};
use odra::schema::{SchemaCustomTypes, SchemaEntrypoints, SchemaEvents};
use odra::{contract_def::HasIdent, host::HostEnv, OdraContract};

use crate::{
    cmd::INSPECT_SUBCOMMAND, custom_types::CustomTypeSet, types, DeployedContractsContainer
};

use super::OdraCommand;

/// Prints a contract's entry points, their arguments and return types, plus its custom types and
/// events — read straight from the schema, so it works without anything being deployed.
#[derive(Default)]
pub(crate) struct InspectCmd {
    contracts: Vec<ContractSchema>
}

impl InspectCmd {
    pub fn add_contract<T: SchemaEntrypoints + SchemaCustomTypes + SchemaEvents + OdraContract>(
        &mut self
    ) {
        self.contracts.push(ContractSchema::new::<T>(None));
    }

    pub fn add_contract_named<
        T: SchemaEntrypoints + SchemaCustomTypes + SchemaEvents + OdraContract
    >(
        &mut self,
        key_name: String
    ) {
        self.contracts
            .push(ContractSchema::new::<T>(Some(key_name)));
    }

    fn available(&self) -> String {
        self.contracts
            .iter()
            .map(|c| c.key_name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl OdraCommand for InspectCmd {
    fn run(
        &self,
        _env: &HostEnv,
        args: &ArgMatches,
        _types: &CustomTypeSet,
        _container: &DeployedContractsContainer
    ) -> Result<()> {
        match args.subcommand() {
            // `inspect <Contract>` — show a single contract.
            Some((name, _)) => {
                let contract = self
                    .contracts
                    .iter()
                    .find(|c| c.key_name == name)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "No contract '{name}' registered. Available: {}",
                            self.available()
                        )
                    })?;
                contract.render();
            }
            // `inspect` — show every registered contract.
            None => {
                if self.contracts.is_empty() {
                    prettycli::info("No contracts registered in the CLI builder.");
                }
                for contract in &self.contracts {
                    contract.render();
                }
            }
        }
        Ok(())
    }
}

impl From<&InspectCmd> for Command {
    fn from(value: &InspectCmd) -> Self {
        let subcommands = value.contracts.iter().map(|c| {
            Command::new(&c.key_name).about(format!("Inspect the {} contract", c.key_name))
        });
        Command::new(INSPECT_SUBCOMMAND)
            .about("Prints the entry points, arguments and types of a contract")
            .subcommands(subcommands)
    }
}

/// The schema slice needed to describe a single contract.
struct ContractSchema {
    key_name: String,
    ident: String,
    entry_points: Vec<Entrypoint>,
    custom_types: BTreeSet<CustomType>
}

impl ContractSchema {
    fn new<T: SchemaEntrypoints + SchemaCustomTypes + SchemaEvents + OdraContract>(
        key_name: Option<String>
    ) -> Self {
        let ident = T::HostRef::ident();
        let key_name = key_name.unwrap_or_else(|| ident.clone());

        let mut custom_types = BTreeSet::new();
        custom_types.extend(T::schema_types().into_iter().flatten());
        custom_types.extend(<T as SchemaEvents>::custom_types().into_iter().flatten());

        Self {
            key_name,
            ident,
            entry_points: T::schema_entrypoints(),
            custom_types
        }
    }

    fn render(&self) {
        prettycli::info(&format!("Contract: {} ({})", self.key_name, self.ident));

        prettycli::info("Entry points:");
        for ep in &self.entry_points {
            let kind = if ep.is_mutable { "mut " } else { "view" };
            let args = ep
                .arguments
                .iter()
                .map(|a| format!("{}: {}", a.name, types::format_type_hint(&a.ty.0)))
                .collect::<Vec<_>>()
                .join(", ");
            let ret = types::format_type_hint(&ep.return_ty.0);
            prettycli::info(&format!("  [{kind}] {}({args}) -> {ret}", ep.name));
            if let Some(desc) = ep.description.as_deref().filter(|d| !d.is_empty()) {
                prettycli::info(&format!("           {desc}"));
            }
        }

        if !self.custom_types.is_empty() {
            prettycli::info("Types & events:");
            for ct in &self.custom_types {
                prettycli::info(&format!("  {}", ct.name()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestContract;

    #[test]
    fn builds_inspect_command_with_subcommands() {
        let mut cmd = InspectCmd::default();
        cmd.add_contract::<TestContract>();

        let clap_cmd: Command = (&cmd).into();
        assert_eq!(clap_cmd.get_name(), INSPECT_SUBCOMMAND);
        assert!(clap_cmd
            .get_subcommands()
            .any(|c| c.get_name() == "TestContract"));
    }

    #[test]
    fn registers_contract_schema() {
        let mut cmd = InspectCmd::default();
        cmd.add_contract::<TestContract>();
        assert_eq!(cmd.contracts.len(), 1);
        assert_eq!(cmd.contracts[0].key_name, "TestContract");
        assert!(!cmd.contracts[0].entry_points.is_empty());
    }
}
