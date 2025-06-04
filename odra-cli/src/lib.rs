//! A rust library for building command line interfaces for Odra smart contracts.
//!
//! The Odra CLI is a command line interface built on top of the [clap] crate
//! that allows users to interact with smart contracts.

#![feature(box_patterns, error_generic_member_access)]
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::str::FromStr;

use clap::{command, Arg, Command};
use cmd::{OdraCliCommand, OdraCommand};
use deploy::DeployScript;
use odra::entry_point_callback::EntryPointsCaller;
use odra::host::Deployer;
use odra::schema::SchemaEvents;
use odra::schema::{casper_contract_schema::CustomType, SchemaCustomTypes, SchemaEntrypoints};
use odra::{
    contract_def::HasIdent,
    host::{EntryPointsCallerProvider, HostEnv},
    OdraContract
};

mod args;
mod cmd;
mod container;
mod entry_point;
#[cfg(test)]
mod test_utils;
mod types;

pub use args::CommandArg;
pub use cmd::scenario::{ScenarioArgs, ScenarioError};
pub use container::DeployedContractsContainer;
use scenario::{Scenario, ScenarioMetadata};

use crate::args::ARG_CONTRACTS;

const CONTRACTS_SUBCOMMAND: &str = "contract";
const SCENARIOS_SUBCOMMAND: &str = "scenario";
const DEPLOY_SUBCOMMAND: &str = "deploy";
const PRINT_EVENTS_SUBCOMMAND: &str = "print-events";

/// Trait that extends the functionality of OdraContract to include deployment capabilities.
pub trait DeployerExt: Sized {
    /// Contract that implements OdraContract and Deployer for Self
    type Contract: OdraContract + 'static + Deployer<Self::Contract>;

    /// Load an existing contract instance from container or deploy a new one.
    fn load_or_deploy(
        env: &HostEnv,
        args: <<Self as DeployerExt>::Contract as OdraContract>::InitArgs,
        container: &mut DeployedContractsContainer,
        gas: u64
    ) -> Result<<<Self as DeployerExt>::Contract as OdraContract>::HostRef, crate::deploy::Error>
    {
        if let Ok(contract) = container.get_ref::<Self::Contract>(env) {
            Ok(contract)
        } else {
            env.set_gas(gas);
            let contract = Self::Contract::try_deploy(env, args)?;
            container.add_contract(&contract)?;
            Ok(contract)
        }
    }
}

impl<T: OdraContract + Deployer<T> + 'static> DeployerExt for T {
    type Contract = T;
}

pub(crate) type CustomTypeSet = BTreeSet<CustomType>;

pub mod scenario {
    //! Traits and structs for defining custom scenarios.
    //!
    //! A scenario is a user-defined set of actions that can be run in the Odra CLI.
    //! If you want to run a custom scenario that calls multiple entry points,
    //! you need to implement the [Scenario] and [ScenarioMetadata] traits.
    pub use crate::cmd::scenario::{
        Scenario, ScenarioArgs as Args, ScenarioError as Error, ScenarioMetadata
    };
}

pub mod deploy {
    //! Traits and structs for defining deploy scripts.
    //!
    //! In a deploy script, you can define the contracts that you want to deploy to the blockchain
    //! and write metadata to the container.
    pub use crate::cmd::deploy::{DeployError as Error, DeployScript};
}

/// Command line interface for Odra smart contracts.
pub struct OdraCli {
    main_cmd: Command,
    scenarios_cmd: Command,
    contracts_cmd: Command,
    print_events_cmd: Command,
    commands: Vec<OdraCliCommand>,
    custom_types: CustomTypeSet,
    host_env: HostEnv,
    callers: HashMap<String, EntryPointsCaller>
}

impl Default for OdraCli {
    fn default() -> Self {
        Self::new()
    }
}

impl OdraCli {
    /// Creates a new empty instance of the Odra CLI.
    pub fn new() -> Self {
        let contracts_cmd = Command::new(CONTRACTS_SUBCOMMAND)
            .about("Commands for interacting with contracts")
            .subcommand_required(true)
            .arg_required_else_help(true);
        let scenarios_cmd = Command::new(SCENARIOS_SUBCOMMAND)
            .about("Commands for running user-defined scenarios")
            .subcommand_required(true)
            .arg_required_else_help(true);
        let print_events_cmd = Command::new("print-events")
            .about("Prints events emitted by the contract")
            .arg_required_else_help(true)
            .subcommand_required(true);
        let main_cmd = Command::new("Odra CLI")
            .subcommand_required(true)
            .arg_required_else_help(true);

        Self {
            main_cmd,
            commands: vec![],
            custom_types: CustomTypeSet::new(),
            host_env: odra_casper_livenet_env::env(),
            contracts_cmd,
            scenarios_cmd,
            print_events_cmd,
            callers: HashMap::new()
        }
    }

    /// Sets the description of the CLI
    pub fn about(mut self, about: &str) -> Self {
        self.main_cmd = self.main_cmd.about(about.to_string());
        self
    }

    /// Adds a contract to the CLI.
    ///
    /// Generates a subcommand for the contract with all of its entry points except the `init` entry point.
    /// To call the constructor of the contract, implement and register the [DeployScript].
    pub fn contract<T: SchemaEntrypoints + SchemaCustomTypes + SchemaEvents + OdraContract>(
        mut self
    ) -> Self {
        let contract_name = T::HostRef::ident();
        self.callers.insert(
            contract_name.clone(),
            T::HostRef::entry_points_caller(&self.host_env)
        );
        self.custom_types
            .extend(T::schema_types().into_iter().flatten());
        self.custom_types
            .extend(<T as SchemaEvents>::custom_types().into_iter().flatten());

        // build entry points commands
        let mut contract_cmd = Command::new(&contract_name)
            .about(format!(
                "Commands for interacting with the {} contract",
                &contract_name
            ))
            .subcommand_required(true)
            .arg_required_else_help(true);
        let print_cmd = Command::new(&contract_name)
            .about(format!("Print events of the {} contract", &contract_name));
        for entry_point in T::schema_entrypoints() {
            if entry_point.name == "init" {
                continue;
            }
            let mut ep_cmd = Command::new(&entry_point.name)
                .about(entry_point.description.clone().unwrap_or_default());
            for arg in args::entry_point_args(&entry_point, &self.custom_types) {
                ep_cmd = ep_cmd.arg(arg);
            }
            // For a payable entry point, a user can attach a value to the call.
            ep_cmd = ep_cmd.arg(args::attached_value_arg());
            // If the entry point is mutable, a transaction is being sent, so we need to
            // provide the gas argument.
            if entry_point.is_mutable {
                ep_cmd = ep_cmd.arg(args::gas_arg());
            }
            contract_cmd = contract_cmd.subcommand(ep_cmd);
        }
        self.contracts_cmd = self.contracts_cmd.subcommand(contract_cmd);
        self.print_events_cmd = self.print_events_cmd.subcommand(print_cmd);

        // store a command
        self.commands
            .push(OdraCliCommand::new_contract::<T>(contract_name.clone()));
        self.commands
            .push(OdraCliCommand::new_print_events(contract_name));
        self
    }

    /// Adds a deploy script to the CLI.
    ///
    /// There is only one deploy script allowed in the CLI.
    pub fn deploy(mut self, script: impl DeployScript + 'static) -> Self {
        // register a subcommand for the deploy script
        self.main_cmd = self
            .main_cmd
            .subcommand(command!(DEPLOY_SUBCOMMAND).about("Runs the deploy script"));
        // store a command
        self.commands.push(OdraCliCommand::new_deploy(script));
        self
    }

    /// Adds a scenario to the CLI.
    ///
    /// Scenarios are user-defined commands that can be run from the CLI. If there
    /// is a complex set of commands that need to be run in a specific order, a
    /// scenario can be used to group them together.
    pub fn scenario<S: ScenarioMetadata + Scenario>(mut self, scenario: S) -> Self {
        // register a subcommand for the scenario
        let mut scenario_cmd = Command::new(S::NAME).about(S::DESCRIPTION);
        let args = scenario
            .args()
            .into_iter()
            .map(Into::into)
            .collect::<Vec<Arg>>();
        for arg in args {
            scenario_cmd = scenario_cmd.arg(arg);
        }

        self.scenarios_cmd = self.scenarios_cmd.subcommand(scenario_cmd);

        // store a command
        self.commands.push(OdraCliCommand::new_scenario(scenario));
        self
    }

    /// Builds the CLI.
    pub fn build(mut self) -> Self {
        self.main_cmd = self.main_cmd.subcommand(self.contracts_cmd.clone());
        self.main_cmd = self.main_cmd.subcommand(self.scenarios_cmd.clone());
        self.main_cmd = self.main_cmd.subcommand(self.print_events_cmd.clone());
        self.main_cmd = self.main_cmd.arg(args::contracts_arg());
        self
    }

    /// Runs the CLI and parses the input.
    pub fn run(self) {
        let matches = self.main_cmd.get_matches();
        // Check if the user provided a custom contracts path.
        let path = args::read(&matches, ARG_CONTRACTS, PathBuf::from_str).ok();
        // Init contracts container with the provided path or default to the resources directory.
        let container = DeployedContractsContainer::new(path.clone())
            .expect("Failed to create or load the deployed contracts container");
        // Register the contracts from the container in the host environment.
        for (name, address) in container.contracts() {
            let caller = self
                .callers
                .get(&name)
                .unwrap_or_else(|| panic!("Caller for {} not found", &name))
                .clone();
            self.host_env.register_contract(address, name, caller);
        }

        let (cmd, args) = matches.subcommand().expect("No subcommand found");

        let (cmd, args) = match cmd {
            DEPLOY_SUBCOMMAND => (
                find_cmd_by_type(&self.commands, DEPLOY_SUBCOMMAND, ""),
                args
            ),
            _ => {
                let (name, ep_matches) = args
                    .subcommand()
                    .unwrap_or_else(|| panic!("No {} subcommand found", cmd));
                (find_cmd_by_type(&self.commands, cmd, name), ep_matches)
            }
        };

        match cmd.run(&self.host_env, args, &self.custom_types, path) {
            Ok(_) => prettycli::info("Command executed successfully"),
            Err(err) => prettycli::error(&format!("{:?}", err))
        }
    }
}

fn find_cmd_by_type<'a>(
    commands: &'a [OdraCliCommand],
    ty: &str,
    name: &str
) -> &'a OdraCliCommand {
    commands
        .iter()
        .find(|cmd| match cmd {
            OdraCliCommand::Deploy(_) if ty == DEPLOY_SUBCOMMAND => true,
            OdraCliCommand::Scenario(scenario)
                if ty == SCENARIOS_SUBCOMMAND && scenario.name() == name =>
            {
                true
            }
            OdraCliCommand::Contract(contract)
                if ty == CONTRACTS_SUBCOMMAND && contract.name() == name =>
            {
                true
            }
            OdraCliCommand::PrintEvents(cmd)
                if ty == PRINT_EVENTS_SUBCOMMAND && cmd.name() == name =>
            {
                true
            }
            _ => false
        })
        .unwrap_or_else(|| {
            panic!(
                "Command for '{}' with type '{}' not found. Make sure the command is registered.",
                name, ty
            )
        })
}
