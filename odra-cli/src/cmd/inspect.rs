use std::collections::BTreeSet;

use anyhow::Result;
use clap::{ArgMatches, Command};
use odra::schema::casper_contract_schema::{CustomType, Entrypoint};
use odra::schema::{SchemaCustomTypes, SchemaEntrypoints, SchemaEvents};
use odra::{contract_def::HasIdent, host::HostEnv, OdraContract};
use serde_derive::Serialize;

use crate::cmd::CmdOutput;
use crate::log;
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
    type Output = ContractsSchemaReport;

    fn exec(
        &self,
        _env: &HostEnv,
        args: &ArgMatches,
        _types: &CustomTypeSet,
        _container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        match args.subcommand() {
            // `inspect <Contract>` — show a single contract (a JSON object).
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
                Ok(Self::Output {
                    contracts: vec![contract.report()]
                })
            }
            None => Ok(Self::Output {
                contracts: self.contracts.iter().map(|c| c.report()).collect()
            })
        }
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

    /// Builds the serializable view of this contract's schema, reusing the same type formatting as
    /// the human renderer so both outputs stay in sync.
    fn report(&self) -> ContractSchemaReport {
        let entry_points = self
            .entry_points
            .iter()
            .map(|ep| EntryPointReport {
                name: ep.name.clone(),
                mutability: if ep.is_mutable { "mutable" } else { "view" },
                arguments: ep
                    .arguments
                    .iter()
                    .map(|a| ArgReport {
                        name: a.name.clone(),
                        ty: types::format_type_hint(&a.ty.0)
                    })
                    .collect(),
                return_type: types::format_type_hint(&ep.return_ty.0),
                description: ep.description.clone().filter(|d| !d.is_empty())
            })
            .collect();

        ContractSchemaReport {
            key_name: self.key_name.clone(),
            ident: self.ident.clone(),
            entry_points,
            types: self
                .custom_types
                .iter()
                .map(|ct| ct.name().to_string())
                .collect()
        }
    }
}

/// Serializable view of a contract's schema for `--json` output.
#[derive(Serialize)]
struct ContractSchemaReport {
    key_name: String,
    ident: String,
    entry_points: Vec<EntryPointReport>,
    types: Vec<String>
}

#[derive(Serialize)]
pub(crate) struct ContractsSchemaReport {
    contracts: Vec<ContractSchemaReport>
}

impl CmdOutput for ContractsSchemaReport {
    fn pretty_print(&self) {
        if self.contracts.is_empty() {
            log("No contracts registered in the CLI builder.");
            return;
        }

        for c in &self.contracts {
            log(format!("Contract: {} ({})", c.key_name, c.ident));

            log("Entry points:");
            for ep in &c.entry_points {
                let kind = ep.mutability;
                let args = ep
                    .arguments
                    .iter()
                    .map(|a| format!("{}: {}", a.name, a.ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret = &ep.return_type;
                log(format!("  [{kind}] {}({args}) -> {ret}", ep.name));
                if let Some(desc) = ep.description.as_deref().filter(|d| !d.is_empty()) {
                    log(format!("           {desc}"));
                }
            }

            if !c.types.is_empty() {
                log("Types & events:");
                for ct in &c.types {
                    log(format!("  {}", ct));
                }
            }
        }
    }
}

#[derive(Serialize)]
struct EntryPointReport {
    name: String,
    mutability: &'static str,
    arguments: Vec<ArgReport>,
    return_type: String,
    description: Option<String>
}

#[derive(Serialize)]
struct ArgReport {
    name: String,
    #[serde(rename = "type")]
    ty: String
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
    fn report_describes_entry_points() {
        let mut cmd = InspectCmd::default();
        cmd.add_contract::<TestContract>();

        let report = cmd.contracts[0].report();
        assert_eq!(report.key_name, "TestContract");
        assert!(!report.entry_points.is_empty());
        assert!(report
            .entry_points
            .iter()
            .all(|e| e.mutability == "mutable" || e.mutability == "view"));
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
