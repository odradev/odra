//! A rust library for building command line interfaces for Odra smart contracts.
//!
//! The Odra CLI is a command line interface built on top of the [clap] crate
//! that allows users to interact with smart contracts.

#![feature(box_patterns, error_generic_member_access)]
use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use clap::ArgMatches;
use cmd::OdraCommand;
use deploy::DeployScript;
use odra::entry_point_callback::EntryPointsCaller;
use odra::host::Deployer;
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

pub use cmd::scenario::{ScenarioArgs, ScenarioError};
pub use container::DeployedContractsContainer;
use scenario::{Scenario, ScenarioMetadata};

use crate::cmd::contract::ContractsCmd;
use crate::cmd::deploy::DeployCmd;
use crate::cmd::events::PrintEventsCmd;
use crate::cmd::main::MainCmd;
use crate::cmd::scenario::ScenariosCmd;
use crate::custom_types::{CustomTypeSet, CustomTypes};

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

        // Init contracts container with the provided path or default to the resources directory.
        let container = match DeployedContractsContainer::new(contracts_path.clone()) {
            Ok(c) => c,
            Err(e) => {
                prettycli::error(&format!("Container error: {e}"));
                return;
            }
        };

        // Register the contracts from the container in the host environment.
        for (name, address) in container.contracts() {
            let caller = self
                .callers
                .get(&name)
                .unwrap_or_else(|| panic!("Caller for {} not found", &name))
                .clone();
            self.host_env.register_contract(address, name, caller);
        }

        let result = match cmd.as_str() {
            DEPLOY_SUBCOMMAND => self.run_command(
                self.deploy_cmd.as_ref().unwrap_or_else(|| {
                    panic!("Deploy command not found. Did you forget to add it?")
                }),
                args,
                contracts_path
            ),
            CONTRACTS_SUBCOMMAND => self.run_command(&self.contracts_cmd, args, contracts_path),
            PRINT_EVENTS_SUBCOMMAND => {
                self.run_command(&self.print_events_cmd, args, contracts_path)
            }
            SCENARIOS_SUBCOMMAND => self.run_command(&self.scenarios_cmd, args, contracts_path),
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
        contracts_path: Option<PathBuf>
    ) -> Result<()> {
        cmd.run(&self.host_env, &args, &self.custom_types, contracts_path)
    }
}
