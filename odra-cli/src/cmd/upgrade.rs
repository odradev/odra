use std::collections::BTreeSet;

use anyhow::Result;
use clap::{ArgAction, ArgMatches, Command};
use odra::casper_types::RuntimeArgs;
use odra::host::{Deployer, HostEnv, UpgradeConfig};
use odra::prelude::{Address, Addressable, OdraError, OdraResult};
use odra::schema::casper_contract_schema::{Access, CustomType, Entrypoint, NamedCLType, Type};
use odra::schema::{SchemaCustomTypes, SchemaEntrypoints, SchemaEvents};
use odra::VmError;
use odra::{contract_def::HasIdent, OdraContract};
use odra_schema::codec::CustomTypeSet;
use serde_derive::Serialize;

use crate::cmd::args::{read_arg, Arg};
use crate::cmd::{CmdOutput, UPGRADE_SUBCOMMAND};
use crate::container::ContractProvider;
use crate::{entry_point, log, DeployedContractsContainer};

use super::OdraCommand;

const ARG_FORCE_CREATE_UPGRADE_GROUP: &str = "force-create-upgrade-group";
const ARG_ALLOW_KEY_OVERRIDE: &str = "allow-key-override";
const ARG_PACKAGE_NAMED_KEY: &str = "package-named-key";

/// Upgrades a deployed contract with the code of `wasm/<Contract>.wasm`, given the contract's
/// address, its upgrade arguments and the upgrade configuration.
type Upgrader = fn(&HostEnv, Address, RuntimeArgs, UpgradeConfig) -> OdraResult<Address>;

/// UpgradeCmd is a struct that represents the upgrade command in the Odra CLI.
///
/// The upgrade command upgrades a contract from the contracts file, calling its `upgrade` entry
/// point with the arguments given on the command line.
#[derive(Default)]
pub(crate) struct UpgradeCmd {
    contracts: Vec<ContractUpgradeCmd>
}

impl UpgradeCmd {
    pub fn add_contract<T>(&mut self)
    where
        T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents
    {
        let package_named_key = <T::HostRef as HasIdent>::contract_name();
        self.contracts.push(ContractUpgradeCmd::new::<T>(
            T::HostRef::ident(),
            package_named_key
        ));
    }

    pub fn add_contract_named<T>(&mut self, package_name: String)
    where
        T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents
    {
        self.contracts.push(ContractUpgradeCmd::new::<T>(
            package_name.clone(),
            package_name
        ));
    }
}

impl OdraCommand for UpgradeCmd {
    type Output = UpgradeReport;

    fn exec(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        container: &DeployedContractsContainer
    ) -> Result<Self::Output> {
        let (package_name, contract_args) = args
            .subcommand()
            .ok_or_else(|| anyhow::anyhow!("No contract found"))?;
        let contract = self
            .contracts
            .iter()
            .find(|cmd| cmd.package_name == package_name)
            .ok_or_else(|| anyhow::anyhow!("No contract found"))?;
        contract.exec(env, contract_args, types, container)
    }
}

impl From<&UpgradeCmd> for Command {
    fn from(value: &UpgradeCmd) -> Self {
        Command::new(UPGRADE_SUBCOMMAND)
            .about("Upgrades a deployed contract with its wasm from the wasm directory")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommands(&value.contracts)
    }
}

/// The upgrade of a single contract.
struct ContractUpgradeCmd {
    package_name: String,
    /// The default of `--package-named-key`.
    package_named_key: String,
    /// The `upgrade` entry point of the contract, with no arguments if the contract has none.
    entry_point: Entrypoint,
    custom_types: BTreeSet<CustomType>,
    upgrader: Upgrader
}

impl ContractUpgradeCmd {
    fn new<T>(package_name: String, package_named_key: String) -> Self
    where
        T: SchemaEntrypoints + OdraContract + SchemaCustomTypes + SchemaEvents
    {
        let entry_point = T::schema_entrypoints()
            .into_iter()
            .find(|entry_point| entry_point.name == "upgrade")
            .unwrap_or_else(no_args_upgrade_entry_point);
        let mut custom_types = BTreeSet::new();
        custom_types.extend(T::schema_types().into_iter().flatten());
        custom_types.extend(<T as SchemaEvents>::custom_types().into_iter().flatten());
        let upgrader: Upgrader = |env, address, args, cfg| {
            T::try_upgrade_with_runtime_args(env, address, args, cfg)
                .map(|contract| contract.address())
        };
        ContractUpgradeCmd {
            package_name,
            package_named_key,
            entry_point,
            custom_types,
            upgrader
        }
    }
}

impl OdraCommand for ContractUpgradeCmd {
    type Output = UpgradeReport;

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
        let upgrade_args = entry_point::runtime_args::compose(&self.entry_point, args, types)?;
        let cfg = UpgradeConfig {
            package_named_key: args
                .get_one::<String>(ARG_PACKAGE_NAMED_KEY)
                .cloned()
                .unwrap_or_else(|| self.package_named_key.clone()),
            force_create_upgrade_group: args.get_flag(ARG_FORCE_CREATE_UPGRADE_GROUP),
            allow_key_override: args
                .get_one::<bool>(ARG_ALLOW_KEY_OVERRIDE)
                .copied()
                .unwrap_or(true)
        };
        let gas = read_arg(args, Arg::Gas).ok_or_else(|| {
            anyhow::anyhow!("Failed to read gas value. Use --gas <value> to specify it.")
        })?;

        env.set_gas(gas);
        let result = (self.upgrader)(env, address, upgrade_args, cfg);
        // Set back to 0 to avoid sending a huge amount of gas in consecutive calls.
        env.set_gas(0);
        let address = result.map_err(|e| {
            anyhow::anyhow!(
                "Upgrading {} failed with: {}",
                self.package_name,
                match e {
                    OdraError::VmError(VmError::Other(msg)) => msg,
                    _ => format!("{:?}", e)
                }
            )
        })?;

        Ok(UpgradeReport {
            contract: self.package_name.clone(),
            package_hash: address.to_string()
        })
    }
}

impl From<&ContractUpgradeCmd> for Command {
    fn from(value: &ContractUpgradeCmd) -> Self {
        let about = value
            .entry_point
            .description
            .clone()
            .unwrap_or_else(|| format!("Upgrades the {} contract", value.package_name));
        Command::new(&value.package_name)
            .about(about)
            .args(entry_point::cmd_args::entry_point_args(
                &value.entry_point,
                &value.custom_types
            ))
            .arg(Arg::Gas)
            .arg(
                clap::Arg::new(ARG_FORCE_CREATE_UPGRADE_GROUP)
                    .long(ARG_FORCE_CREATE_UPGRADE_GROUP)
                    .help(
                        "Create the upgrade group of the contract. Needed for contracts deployed \
                         with Odra 2.2 or earlier, or not with Odra at all"
                    )
                    .action(ArgAction::SetTrue)
            )
            .arg(
                clap::Arg::new(ARG_ALLOW_KEY_OVERRIDE)
                    .long(ARG_ALLOW_KEY_OVERRIDE)
                    .help(
                        "Overwrite the named key the package hash is stored under. Set it to \
                         false to fail if the key already exists"
                    )
                    .value_name("BOOL")
                    .num_args(0..=1)
                    .default_value("true")
                    .default_missing_value("true")
                    .value_parser(clap::value_parser!(bool))
                    .action(ArgAction::Set)
            )
            .arg(
                clap::Arg::new(ARG_PACKAGE_NAMED_KEY)
                    .long(ARG_PACKAGE_NAMED_KEY)
                    .help(format!(
                        "The package hash is stored under the `<NAME>_package_hash` named key \
                         [default: {}]",
                        value.package_named_key
                    ))
                    .value_name("NAME")
                    .action(ArgAction::Set)
            )
    }
}

fn no_args_upgrade_entry_point() -> Entrypoint {
    Entrypoint {
        name: "upgrade".to_string(),
        description: None,
        is_mutable: true,
        is_payable: false,
        arguments: vec![],
        return_ty: Type(NamedCLType::Unit),
        is_contract_context: true,
        access: Access::Public
    }
}

/// The result of an upgrade: the contract and its package hash, which an upgrade keeps.
#[derive(Serialize)]
pub(crate) struct UpgradeReport {
    contract: String,
    package_hash: String
}

impl CmdOutput for UpgradeReport {
    fn pretty_print(&self) {
        log(format!(
            "Contract {} upgraded, package hash: {}",
            self.contract, self.package_hash
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use odra::casper_types::{CLValue, RuntimeArgs};

    use super::*;
    use crate::test_utils::{self, TestContract};

    const PACKAGE_HASH: &str =
        "hash-0101010101010101010101010101010101010101010101010101010101010101";

    thread_local! {
        static UPGRADES: RefCell<Vec<(Address, RuntimeArgs, UpgradeConfig)>> =
            const { RefCell::new(Vec::new()) };
    }

    fn mock_upgrader(
        _env: &HostEnv,
        address: Address,
        args: RuntimeArgs,
        cfg: UpgradeConfig
    ) -> OdraResult<Address> {
        UPGRADES.with(|upgrades| upgrades.borrow_mut().push((address, args, cfg)));
        Ok(address)
    }

    fn upgrade_cmd() -> UpgradeCmd {
        let mut cmd = UpgradeCmd::default();
        cmd.add_contract::<TestContract>();
        cmd.add_contract_named::<TestContract>("SecondTestContract".to_string());
        for contract in cmd.contracts.iter_mut() {
            contract.upgrader = mock_upgrader;
        }
        cmd
    }

    fn container() -> DeployedContractsContainer {
        let path = std::env::temp_dir().join(format!(
            "odra-cli-upgrade-{}-{:?}.toml",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::write(
            &path,
            format!(
                "time = \"2026-11-03\"\n\n[[contracts]]\nname = \"TestContract\"\npackage_name = \"TestContract\"\npackage_hash = \"{PACKAGE_HASH}\"\n"
            )
        )
        .unwrap();
        crate::utils::container_from_file(path).unwrap()
    }

    fn run(argv: &[&str]) -> Result<(Address, RuntimeArgs, UpgradeConfig)> {
        let cmd = upgrade_cmd();
        let args = Command::from(&cmd).try_get_matches_from(argv)?;
        let types = CustomTypeSet::new();
        let env = test_utils::mock_host_env();
        cmd.exec(&env, &args, &types, &container())?;
        Ok(UPGRADES
            .with(|upgrades| upgrades.borrow_mut().pop())
            .unwrap())
    }

    #[test]
    fn upgrade_cmd_lists_contracts() {
        let cmd = upgrade_cmd();
        let clap_cmd: Command = (&cmd).into();
        assert_eq!(clap_cmd.get_name(), UPGRADE_SUBCOMMAND);
        let names = clap_cmd
            .get_subcommands()
            .map(|c| c.get_name().to_string())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["TestContract", "SecondTestContract"]);
    }

    #[test]
    fn upgrade_args_and_gas_are_required() {
        let cmd = upgrade_cmd();
        for argv in [
            vec!["upgrade", "TestContract", "--version", "2"],
            vec!["upgrade", "TestContract", "--gas", "10000000000"]
        ] {
            let result = Command::from(&cmd).try_get_matches_from(argv);
            assert_eq!(
                result.unwrap_err().kind(),
                clap::error::ErrorKind::MissingRequiredArgument
            );
        }
    }

    #[test]
    fn upgrade_with_default_config() {
        let (address, args, cfg) = run(&[
            "upgrade",
            "TestContract",
            "--gas",
            "10000000000",
            "--version",
            "2"
        ])
        .unwrap();
        assert_eq!(address.to_string(), PACKAGE_HASH);
        assert_eq!(args.get("version"), Some(&CLValue::from_t(2u32).unwrap()));
        assert_eq!(args.len(), 1);
        assert_eq!(cfg.package_named_key, "TestContract");
        assert!(!cfg.force_create_upgrade_group);
        assert!(cfg.allow_key_override);
    }

    #[test]
    fn upgrade_with_custom_config() {
        let (_, _, cfg) = run(&[
            "upgrade",
            "TestContract",
            "--gas",
            "10000000000",
            "--version",
            "2",
            "--force-create-upgrade-group",
            "--allow-key-override",
            "false",
            "--package-named-key",
            "Custom"
        ])
        .unwrap();
        assert_eq!(cfg.package_named_key, "Custom");
        assert!(cfg.force_create_upgrade_group);
        assert!(!cfg.allow_key_override);

        // The flag alone allows the override.
        let (_, _, cfg) = run(&[
            "upgrade",
            "TestContract",
            "--gas",
            "10000000000",
            "--version",
            "2",
            "--allow-key-override"
        ])
        .unwrap();
        assert!(cfg.allow_key_override);
    }

    #[test]
    fn upgrade_fails_for_a_contract_not_deployed() {
        let err = run(&[
            "upgrade",
            "SecondTestContract",
            "--gas",
            "10000000000",
            "--version",
            "2"
        ])
        .err()
        .expect("the contract is not in the contracts file");
        assert!(
            err.to_string()
                .contains("Contract `SecondTestContract` not found"),
            "{err}"
        );
    }

    #[test]
    fn named_contract_defaults_to_its_package_name_as_the_key() {
        let cmd = upgrade_cmd();
        assert_eq!(cmd.contracts[1].package_named_key, "SecondTestContract");
    }

    #[test]
    fn contract_without_upgrade_takes_no_args() {
        let entry_point = no_args_upgrade_entry_point();
        assert!(entry_point.arguments.is_empty());
        assert!(entry_point.is_mutable);
    }
}
