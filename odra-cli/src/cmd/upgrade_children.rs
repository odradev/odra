use std::collections::BTreeSet;

use anyhow::Result;
use clap::{ArgAction, ArgMatches, Command};
use odra::casper_types::bytesrepr::{Bytes, FromBytes, ToBytes};
use odra::casper_types::{CLValue, RuntimeArgs};
use odra::host::HostEnv;
use odra::prelude::{Address, BTreeMap, OdraError};
use odra::schema::casper_contract_schema::{CustomType, Entrypoint};
use odra::schema::{SchemaCustomTypes, SchemaEntrypoints, SchemaEvents};
use odra::{CallDef, OdraContract, VmError};
use odra_schema::codec::CustomTypeSet;
use serde_derive::Serialize;

use crate::cmd::args::{read_arg, Arg};
use crate::cmd::{CmdOutput, UPGRADE_CHILDREN_SUBCOMMAND};
use crate::container::ContractProvider;
use crate::entry_point::factory::{
    self, BATCH_UPGRADE_CHILD_CONTRACT, CONTRACT_NAME_ARG, UPGRADE_CHILD_CONTRACT
};
use crate::{entry_point, log, DeployedContractsContainer};

use super::OdraCommand;

const BATCH_ARGS_ARG: &str = "args";
const ARG_BATCH_SIZE: &str = "batch-size";

/// UpgradeChildrenCmd is a struct that represents the upgrade-children command in the Odra CLI.
///
/// The command upgrades every child contract of a factory (`#[odra::module(factory = on)]`) from
/// the contracts file with the factory's current code, with `batch_upgrade_child_contract` calls:
/// one, or one per `--batch-size` children. The children are the contracts the factory announced
/// with its `ContractDeployed` events.
#[derive(Default)]
pub(crate) struct UpgradeChildrenCmd {
    factories: Vec<FactoryUpgradeChildrenCmd>
}

impl UpgradeChildrenCmd {
    /// Adds the contract if it is a factory, ignores it otherwise.
    pub fn add_contract<T>(&mut self, package_name: String)
    where
        T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents
    {
        if let Some(cmd) = FactoryUpgradeChildrenCmd::new::<T>(package_name) {
            self.factories.push(cmd);
        }
    }

    /// Whether any factory was added: without one the command is not listed.
    pub fn is_empty(&self) -> bool {
        self.factories.is_empty()
    }
}

impl OdraCommand for UpgradeChildrenCmd {
    type Output = UpgradeChildrenReport;

    fn exec(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        let (package_name, factory_args) = args
            .subcommand()
            .ok_or_else(|| anyhow::anyhow!("No factory found"))?;
        let factory = self
            .factories
            .iter()
            .find(|cmd| cmd.package_name == package_name)
            .ok_or_else(|| anyhow::anyhow!("No factory found"))?;
        factory.exec(env, factory_args, types, container)
    }
}

impl From<&UpgradeChildrenCmd> for Command {
    fn from(value: &UpgradeChildrenCmd) -> Self {
        Command::new(UPGRADE_CHILDREN_SUBCOMMAND)
            .about("Upgrades all child contracts of a factory with the factory's current code")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommands(&value.factories)
    }
}

/// The upgrade of the children of a single factory.
struct FactoryUpgradeChildrenCmd {
    package_name: String,
    /// The upgrade arguments of a child: the arguments of `upgrade_child_contract` but the name.
    upgrade_args: Entrypoint,
    custom_types: BTreeSet<CustomType>
}

impl FactoryUpgradeChildrenCmd {
    fn new<T>(package_name: String) -> Option<Self>
    where
        T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents
    {
        let entry_points = T::schema_entrypoints();
        if !factory::is_factory(&entry_points) {
            return None;
        }
        let mut upgrade_args = entry_points
            .into_iter()
            .find(|entry_point| entry_point.name == UPGRADE_CHILD_CONTRACT)?;
        upgrade_args
            .arguments
            .retain(|arg| arg.name != CONTRACT_NAME_ARG);
        let mut custom_types = BTreeSet::new();
        custom_types.extend(T::schema_types().into_iter().flatten());
        custom_types.extend(<T as SchemaEvents>::custom_types().into_iter().flatten());
        Some(FactoryUpgradeChildrenCmd {
            package_name,
            upgrade_args,
            custom_types
        })
    }
}

impl OdraCommand for FactoryUpgradeChildrenCmd {
    type Output = UpgradeChildrenReport;

    fn exec(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        let address = container
            .address_by_name(&self.package_name)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Contract `{}` not found in the contracts file. Deploy it first.",
                    self.package_name
                )
            })?;
        let upgrade_args = entry_point::runtime_args::compose(&self.upgrade_args, args, types)?;
        let gas = read_arg(args, Arg::Gas).ok_or_else(|| {
            anyhow::anyhow!("Failed to read gas value. Use --gas <value> to specify it.")
        })?;
        let batch_size = args
            .get_one::<u32>(ARG_BATCH_SIZE)
            .map(|size| *size as usize);

        let events = (0..env.events_count(&address))
            .map(|index| env.get_event_bytes(&address, index))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                anyhow::anyhow!("Reading the events of {} failed: {e:?}", self.package_name)
            })?;
        let children = child_names(&events);
        if children.is_empty() {
            anyhow::bail!("Factory {} has no child contracts", self.package_name);
        }

        let batches = children
            .chunks(batch_size.unwrap_or(children.len()))
            .collect::<Vec<_>>();
        for (index, batch) in batches.iter().enumerate() {
            let call_def = CallDef::new(
                BATCH_UPGRADE_CHILD_CONTRACT,
                true,
                batch_upgrade_args(batch, &upgrade_args)?
            );
            env.set_gas(gas);
            let result = env.raw_call_contract(address, call_def, false);
            // Set back to 0 to avoid sending a huge amount of gas in consecutive calls.
            env.set_gas(0);
            if let Err(e) = result {
                let upgraded = batches[..index].concat();
                anyhow::bail!(
                    "Upgrading the children of {} failed with: {}\nUpgraded: {}\nNot upgraded: {}",
                    self.package_name,
                    match e {
                        OdraError::VmError(VmError::Other(msg)) => msg,
                        _ => format!("{:?}", e)
                    },
                    list_or_none(&upgraded),
                    list_or_none(&batches[index..].concat())
                );
            }
            if batches.len() > 1 {
                log(format!(
                    "Batch {}/{} upgraded: {}",
                    index + 1,
                    batches.len(),
                    batch.join(", ")
                ));
            }
        }

        Ok(UpgradeChildrenReport {
            factory: self.package_name.clone(),
            transactions: batches.len(),
            children
        })
    }
}

impl From<&FactoryUpgradeChildrenCmd> for Command {
    fn from(value: &FactoryUpgradeChildrenCmd) -> Self {
        Command::new(&value.package_name)
            .about(format!(
                "Upgrades all child contracts of the {} factory",
                value.package_name
            ))
            .args(entry_point::cmd_args::entry_point_args(
                &value.upgrade_args,
                &value.custom_types
            ))
            .arg(Arg::Gas)
            .arg(
                clap::Arg::new(ARG_BATCH_SIZE)
                    .long(ARG_BATCH_SIZE)
                    .help(
                        "Upgrade at most N children per transaction, each with the --gas amount \
                         [default: all in one transaction]"
                    )
                    .value_name("N")
                    .value_parser(clap::value_parser!(u32).range(1..))
                    .action(ArgAction::Set)
            )
    }
}

fn list_or_none(names: &[String]) -> String {
    match names.is_empty() {
        true => "none".to_string(),
        false => names.join(", ")
    }
}

/// The names of the children announced by the factory's `<Factory>ContractDeployed` events, in
/// the order they were deployed.
///
/// The factory emits the event again when a child is upgraded, so a name is listed once. The
/// event name is matched by its suffix: a factory upgraded from another module keeps the events
/// of the module it was before.
fn child_names(events: &[Bytes]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for event in events {
        let Ok((event_name, fields)) = String::from_bytes(event) else {
            continue;
        };
        if !(event_name.starts_with("event_") && event_name.ends_with("ContractDeployed")) {
            continue;
        }
        let Ok(((name, _address), rest)) = <(String, Address)>::from_bytes(fields) else {
            continue;
        };
        if rest.is_empty() && !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// The arguments of `batch_upgrade_child_contract`: every child with the same upgrade arguments,
/// and the upgrade configuration the generated factory ref passes along.
fn batch_upgrade_args(children: &[String], upgrade_args: &RuntimeArgs) -> Result<RuntimeArgs> {
    let args_bytes = Bytes::from(upgrade_args.to_bytes()?);
    let batch = children
        .iter()
        .map(|name| (name.clone(), args_bytes.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut args = RuntimeArgs::new();
    args.insert_cl_value(BATCH_ARGS_ARG, CLValue::from_t(batch).map_err(cl_error)?);
    factory::insert_cfg_args(BATCH_UPGRADE_CHILD_CONTRACT, &mut args).map_err(cl_error)?;
    Ok(args)
}

fn cl_error(e: odra::casper_types::CLValueError) -> anyhow::Error {
    anyhow::anyhow!("{e:?}")
}

/// The result of an upgrade of the children of a factory.
#[derive(Serialize)]
pub(crate) struct UpgradeChildrenReport {
    factory: String,
    /// The number of `batch_upgrade_child_contract` transactions sent.
    transactions: usize,
    children: Vec<String>
}

impl CmdOutput for UpgradeChildrenReport {
    fn pretty_print(&self) {
        let transactions = match self.transactions {
            1 => String::new(),
            n => format!(" in {n} transactions")
        };
        log(format!(
            "Upgraded {} child contracts of {}{}: {}",
            self.children.len(),
            self.factory,
            transactions,
            self.children.join(", ")
        ));
    }
}

#[cfg(test)]
mod tests {
    use odra::args::{BatchUpgradeArgs, EntrypointArgument};
    use odra::consts;
    use odra::prelude::Address;

    use super::*;
    use crate::test_utils::{
        TestContract, TestCounterFactory, TestCounterFactoryContractDeployed,
        TestCounterUpgradeArgs
    };

    const CHILD: &str = "hash-0101010101010101010101010101010101010101010101010101010101010101";

    fn cmd() -> UpgradeChildrenCmd {
        let mut cmd = UpgradeChildrenCmd::default();
        cmd.add_contract::<TestContract>("TestContract".to_string());
        cmd.add_contract::<TestCounterFactory>("TestCounterFactory".to_string());
        cmd
    }

    fn deployed(name: &str) -> Bytes {
        let event = TestCounterFactoryContractDeployed {
            contract_name: name.to_string(),
            contract_address: Address::new(CHILD).unwrap()
        };
        Bytes::from(event.to_bytes().unwrap())
    }

    #[test]
    fn only_factories_are_listed() {
        let cmd = cmd();
        assert!(!cmd.is_empty());
        let clap_cmd: Command = (&cmd).into();
        let names = clap_cmd
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["TestCounterFactory"]);

        let mut cmd = UpgradeChildrenCmd::default();
        cmd.add_contract::<TestContract>("TestContract".to_string());
        assert!(cmd.is_empty());
    }

    #[test]
    fn takes_the_child_upgrade_args_but_the_name() {
        let cmd = cmd();
        let parse = |argv: Vec<&str>| Command::from(&cmd).try_get_matches_from(argv);
        assert!(parse(vec![
            "upgrade-children",
            "TestCounterFactory",
            "--gas",
            "10000000000",
            "--new_value",
            "5",
        ])
        .is_ok());
        assert_eq!(
            parse(vec![
                "upgrade-children",
                "TestCounterFactory",
                "--gas",
                "10000000000"
            ])
            .unwrap_err()
            .kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
        assert_eq!(
            parse(vec![
                "upgrade-children",
                "TestCounterFactory",
                "--new_value",
                "5"
            ])
            .unwrap_err()
            .kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
        assert_eq!(
            parse(vec![
                "upgrade-children",
                "TestCounterFactory",
                "--gas",
                "10000000000",
                "--new_value",
                "5",
                "--contract_name",
                "A",
            ])
            .unwrap_err()
            .kind(),
            clap::error::ErrorKind::UnknownArgument
        );
    }

    #[test]
    fn batch_size_is_at_least_one() {
        let cmd = cmd();
        let parse = |batch_size: &str| {
            Command::from(&cmd).try_get_matches_from(vec![
                "upgrade-children",
                "TestCounterFactory",
                "--gas",
                "10000000000",
                "--new_value",
                "5",
                "--batch-size",
                batch_size,
            ])
        };
        let matches = parse("2").unwrap();
        let (_, factory_args) = matches.subcommand().unwrap();
        assert_eq!(factory_args.get_one::<u32>(ARG_BATCH_SIZE), Some(&2));
        assert_eq!(
            parse("0").unwrap_err().kind(),
            clap::error::ErrorKind::ValueValidation
        );
    }

    #[test]
    fn children_are_read_from_the_deployed_events() {
        let other_event = Bytes::from(("event_Transfer".to_string(), 1u32).to_bytes().unwrap());
        // The event of the module the factory was before an upgrade.
        let old_module_event = Bytes::from(
            (
                "event_OldCounterFactoryContractDeployed".to_string(),
                "Old".to_string(),
                Address::new(CHILD).unwrap()
            )
                .to_bytes()
                .unwrap()
        );
        let events = vec![
            deployed("FromTen"),
            other_event,
            deployed("FromTwo"),
            // Emitted again by an upgrade of the child.
            deployed("FromTen"),
            old_module_event,
        ];
        assert_eq!(child_names(&events), vec!["FromTen", "FromTwo", "Old"]);
        assert!(child_names(&[]).is_empty());
    }

    #[test]
    fn batch_args_match_the_typed_batch_upgrade_args() {
        let children = vec!["FromTen".to_string(), "FromTwo".to_string()];
        let upgrade_args: RuntimeArgs = TestCounterUpgradeArgs { new_value: 42 }.into();
        let actual = batch_upgrade_args(&children, &upgrade_args).unwrap();

        let typed = children
            .iter()
            .map(|name| (name.clone(), TestCounterUpgradeArgs { new_value: 42 }))
            .collect::<BTreeMap<_, _>>();
        let mut expected = RuntimeArgs::new();
        // As the generated `batch_upgrade_child_contract` of the factory ref builds them.
        BatchUpgradeArgs::from(typed).insert_runtime_arg(BATCH_ARGS_ARG, &mut expected);
        true.insert_runtime_arg(consts::IS_FACTORY_UPGRADE_ARG, &mut expected);
        true.insert_runtime_arg(consts::ALLOW_KEY_OVERRIDE_ARG, &mut expected);
        false.insert_runtime_arg(consts::CREATE_UPGRADE_GROUP, &mut expected);
        assert_eq!(actual, expected);
    }
}
