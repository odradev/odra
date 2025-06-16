//! A rust library for building command line interfaces for Odra smart contracts.
//!
//! The Odra CLI is a command line interface built on top of the [clap] crate
//! that allows users to interact with smart contracts.

#![feature(box_patterns, error_generic_member_access)]
use std::collections::HashMap;

use anyhow::Result;
use clap::ArgMatches;
use cmd::OdraCommand;
use deploy::DeployScript;
use odra::entry_point_callback::EntryPointsCaller;
use odra::schema::SchemaEvents;
use odra::schema::{SchemaCustomTypes, SchemaEntrypoints};
use odra::{
    contract_def::HasIdent,
    host::{EntryPointsCallerProvider, HostEnv},
    OdraContract
};

mod cmd;
mod container;
mod custom_types;
mod entry_point;
mod parser;
#[cfg(test)]
mod test_utils;
mod types;
mod utils;

pub use cmd::args::CommandArg;
pub use cmd::{ScenarioArgs, ScenarioError};
pub use container::{ContractProvider, DeployedContractsContainer};
pub use utils::{log, DeployerExt};

use crate::cmd::{
    ContractsCmd, DeployCmd, MainCmd, MutableCommand, PrintEventsCmd, ScenariosCmd,
    CONTRACTS_SUBCOMMAND, DEPLOY_SUBCOMMAND, PRINT_EVENTS_SUBCOMMAND, SCENARIOS_SUBCOMMAND
};
use crate::container::FileContractStorage;
use crate::custom_types::{CustomTypeSet, CustomTypes};
use scenario::{Scenario, ScenarioMetadata};

pub mod scenario {
    //! Traits and structs for defining custom scenarios.
    //!
    //! A scenario is a user-defined set of actions that can be run in the Odra CLI.
    //! If you want to run a custom scenario that calls multiple entry points,
    //! you need to implement the [Scenario] and [ScenarioMetadata] traits.
    pub use crate::cmd::{
        Scenario, ScenarioArgs as Args, ScenarioError as Error, ScenarioMetadata
    };
}

pub mod deploy {
    //! Traits and structs for defining deploy scripts.
    //!
    //! In a deploy script, you can define the contracts that you want to deploy to the blockchain
    //! and write metadata to the container.
    pub use crate::cmd::{DeployError as Error, DeployScript};
}

/// Command line interface for Odra smart contracts.
pub struct OdraCli {
    main_cmd: MainCmd,
    deploy_cmd: Option<DeployCmd>,
    contracts_cmd: ContractsCmd,
    print_events_cmd: PrintEventsCmd,
    scenarios_cmd: ScenariosCmd,
    custom_types: CustomTypes,
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
        Self {
            main_cmd: MainCmd::default(),
            deploy_cmd: None,
            contracts_cmd: ContractsCmd::default(),
            print_events_cmd: PrintEventsCmd::default(),
            scenarios_cmd: ScenariosCmd::default(),
            host_env: odra_casper_livenet_env::env(),
            custom_types: CustomTypes::default(),
            callers: HashMap::default()
        }
    }

    #[cfg(test)]
    pub fn test(host_env: HostEnv) -> Self {
        Self {
            main_cmd: MainCmd::default(),
            deploy_cmd: None,
            contracts_cmd: ContractsCmd::default(),
            print_events_cmd: PrintEventsCmd::default(),
            scenarios_cmd: ScenariosCmd::default(),
            host_env,
            custom_types: CustomTypes::default(),
            callers: HashMap::default()
        }
    }

    /// Sets the description of the CLI
    pub fn about(mut self, about: &'static str) -> Self {
        self.main_cmd = self.main_cmd.about(about);
        self
    }

    /// Adds a contract to the CLI.
    ///
    /// Generates a subcommand for the contract with all of its entry points except the `init` entry point.
    /// To call the constructor of the contract, implement and register the [DeployScript].
    pub fn contract<T: SchemaEntrypoints + SchemaCustomTypes + SchemaEvents + OdraContract>(
        mut self
    ) -> Self {
        self.callers.insert(
            T::HostRef::ident(),
            T::HostRef::entry_points_caller(&self.host_env)
        );
        self.custom_types.register::<T>();
        self.contracts_cmd.add_contract::<T>();
        self.print_events_cmd.add_contract::<T>();
        self
    }

    /// Adds a deploy script to the CLI.
    ///
    /// There is only one deploy script allowed in the CLI.
    pub fn deploy(mut self, script: impl DeployScript + 'static) -> Self {
        let cmd = DeployCmd::new(script);
        self.main_cmd = self.main_cmd.subcommand(&cmd);
        self.deploy_cmd = Some(cmd);
        self
    }

    /// Adds a scenario to the CLI.
    ///
    /// Scenarios are user-defined commands that can be run from the CLI. If there
    /// is a complex set of commands that need to be run in a specific order, a
    /// scenario can be used to group them together.
    pub fn scenario<S: ScenarioMetadata + Scenario>(mut self, scenario: S) -> Self {
        self.scenarios_cmd.add_scenario(scenario);
        self
    }

    /// Builds the CLI.
    pub fn build(mut self) -> Self {
        self.main_cmd = self.main_cmd.subcommand(&self.contracts_cmd);
        self.main_cmd = self.main_cmd.subcommand(&self.scenarios_cmd);
        self.main_cmd = self.main_cmd.subcommand(&self.print_events_cmd);
        self
    }

    /// Runs the CLI and parses the input.
    pub fn run(self) {
        let (cmd, args, contracts_path) = self.main_cmd.get_matches();

        let storage = FileContractStorage::new(contracts_path.clone()).unwrap_or_else(|e| {
            prettycli::error(&format!("Failed to create contract storage: {e}"));
            std::process::exit(1);
        });
        // Init contracts container with the provided path or default to the resources directory.
        let mut container = DeployedContractsContainer::instance(storage);

        // Register the contracts from the container in the host environment.
        for (name, address) in container.all_contracts() {
            let caller = self
                .callers
                .get(&name)
                .unwrap_or_else(|| panic!("Caller for {} not found", &name))
                .clone();
            self.host_env.register_contract(address, name, caller);
        }

        let result = match cmd.as_str() {
            DEPLOY_SUBCOMMAND => self
                .deploy_cmd
                .as_ref()
                .unwrap_or_else(|| panic!("Deploy command not found. Did you forget to add it?"))
                .run(&self.host_env, &args, &self.custom_types, &mut container),
            CONTRACTS_SUBCOMMAND => self.run_command(&self.contracts_cmd, args, &container),
            PRINT_EVENTS_SUBCOMMAND => self.run_command(&self.print_events_cmd, args, &container),
            SCENARIOS_SUBCOMMAND => self.run_command(&self.scenarios_cmd, args, &container),
            _ => unreachable!()
        };

        match result {
            Ok(_) => prettycli::info("Command executed successfully"),
            Err(err) => prettycli::error(&format!("{:?}", err))
        }
    }

    fn run_command<T: OdraCommand>(
        &self,
        cmd: &T,
        args: ArgMatches,
        container: &DeployedContractsContainer
    ) -> Result<()> {
        cmd.run(&self.host_env, &args, &self.custom_types, container)
    }
}
