use std::{
    collections::HashMap,
    path::{Path, PathBuf}
};

use anyhow::Result;
use clap::ArgMatches;
use odra::{
    contract_def::HasIdent,
    entry_point_callback::EntryPointsCaller,
    host::{EntryPointsCallerProvider, HostEnv},
    schema::{SchemaCustomTypes, SchemaEntrypoints, SchemaEvents},
    OdraContract
};
use odra_casper_livenet_env::LivenetError;

use crate::{
    cmd::{
        ContractsCmd, DeployCmd, DeployScript, MainCmd, MutableCommand, OdraCommand,
        PrintEventsCmd, Scenario, ScenarioMetadata, ScenariosCmd, WhoamiCmd, CONTRACTS_SUBCOMMAND,
        DEPLOY_SUBCOMMAND, PRINT_EVENTS_SUBCOMMAND, REPL_SUBCOMMAND, SCENARIOS_SUBCOMMAND,
        WHOAMI_SUBCOMMAND
    },
    container::FileContractStorage,
    custom_types::CustomTypes,
    utils::get_default_contracts_file,
    ContractProvider, DeployedContractsContainer
};

mod repl;

/// Command line interface for Odra smart contracts.
pub struct OdraCli {
    main_cmd: MainCmd,
    deploy_cmd: Option<DeployCmd>,
    contracts_cmd: ContractsCmd,
    print_events_cmd: PrintEventsCmd,
    scenarios_cmd: ScenariosCmd,
    whoami_cmd: WhoamiCmd,
    custom_types: CustomTypes,
    host_env: HostEnv,
    callers: HashMap<(String, String), EntryPointsCaller>,
    default_contract_path: Option<PathBuf>
}

impl Default for OdraCli {
    fn default() -> Self {
        Self::new()
    }
}

impl OdraCli {
    /// Creates a new empty instance of the Odra CLI.
    pub fn new() -> Self {
        let host_env = match odra_casper_livenet_env::env_safe() {
            Ok(e) => e,
            Err(LivenetError::EnvVariableNotSet(e)) => {
                prettycli::error(&format!(
                    "Livenet env misconfigured! {} env var is missing.",
                    e
                ));
                prettycli::info("Visit offical docs to read more:");
                prettycli::link("https://odra.dev/docs/backends/livenet#setup");
                std::process::exit(1)
            }
            Err(e) => {
                prettycli::error(&format!("Livent misconfigured: {e:#}"));
                std::process::exit(1)
            }
        };
        Self {
            main_cmd: MainCmd::default(),
            deploy_cmd: None,
            contracts_cmd: ContractsCmd::default(),
            print_events_cmd: PrintEventsCmd::default(),
            scenarios_cmd: ScenariosCmd::default(),
            whoami_cmd: WhoamiCmd::new(),
            host_env,
            custom_types: CustomTypes::default(),
            callers: HashMap::default(),
            default_contract_path: None
        }
    }

    /// Sets the description of the CLI
    pub fn about(mut self, about: &'static str) -> Self {
        self.main_cmd = self.main_cmd.about(about);
        self
    }

    /// Sets the path to a file that stores the deployed contract addresses.
    ///
    /// The path is relative to the resources directory in the project root.
    pub fn contracts_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.default_contract_path = Some(path.as_ref().to_path_buf());
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
            (T::HostRef::ident(), T::HostRef::ident()),
            T::HostRef::entry_points_caller(&self.host_env)
        );
        self.custom_types.register::<T>();
        self.contracts_cmd.add_contract::<T>();
        self.print_events_cmd.add_contract::<T>();
        self
    }

    /// Adds a named contract to the CLI, in case of multiple instances of the same contract.
    ///
    /// Generates a subcommand for the contract with all of its entry points except the `init` entry point.
    /// To call the constructor of the contract, implement and register the [DeployScript].
    pub fn named_contract<
        T: SchemaEntrypoints + SchemaCustomTypes + SchemaEvents + OdraContract
    >(
        mut self,
        name: String
    ) -> Self {
        self.callers.insert(
            (T::HostRef::ident(), name.clone()),
            T::HostRef::entry_points_caller(&self.host_env)
        );
        self.custom_types.register::<T>();
        self.contracts_cmd.add_contract_named::<T>(name.clone());
        self.print_events_cmd.add_contract_named::<T>(name);
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
        self.main_cmd = self.main_cmd.subcommand(&self.whoami_cmd);
        self.main_cmd = self.main_cmd.subcommand(
            clap::Command::new(REPL_SUBCOMMAND)
                .about("Starts an interactive REPL session, keeping the host environment warm across commands.")
        );
        self
    }

    /// Runs the CLI once, parsing the input from the process arguments and exiting on error.
    pub fn run(self) {
        let (cmd, args, contracts_path) = self.main_cmd.get_matches();
        let contracts_path = match contracts_path {
            Some(path) => Some(path),
            None => self.default_contract_path.clone()
        };

        // Init contracts container with the provided path or default to the resources directory.
        let mut container = self
            .load_container(contracts_path.clone())
            .unwrap_or_else(|e| {
                prettycli::error(&format!("{e:#}"));
                std::process::exit(1);
            });

        // Register the contracts from the container in the host environment.
        if let Err(err) = self.register_deployed_contracts(&container, &contracts_path) {
            prettycli::error(&format!("{err:#}"));
            std::process::exit(1);
        }

        if let Err(err) = self.dispatch(&cmd, &args, &mut container) {
            prettycli::error(&format!("{err:#}"));
            std::process::exit(1);
        }
    }

    /// Builds the deployed-contracts container from the given path (or the default location).
    fn load_container(
        &self,
        contracts_path: Option<PathBuf>
    ) -> Result<DeployedContractsContainer> {
        let storage = FileContractStorage::new(contracts_path)
            .map_err(|e| anyhow::anyhow!("Failed to create contract storage: {e}"))?;
        Ok(DeployedContractsContainer::instance(storage))
    }

    /// Registers every deployed contract from the container in the host environment.
    ///
    /// Only contracts that have a matching caller (added via `.contract::<T>()` in the builder) can
    /// be registered. Returns an error naming the missing caller instead of exiting, so the REPL can
    /// report it without killing the session.
    fn register_deployed_contracts(
        &self,
        container: &DeployedContractsContainer,
        contracts_path: &Option<PathBuf>
    ) -> Result<()> {
        for deployed_contract in container.all_contracts() {
            let caller = self
                .callers
                .get(&(deployed_contract.name(), deployed_contract.key_name()))
                .ok_or_else(|| {
                    let path = match contracts_path {
                        Some(path) => path.to_str().map(|s| s.to_string()).unwrap_or_default(),
                        None => get_default_contracts_file()
                    };
                    anyhow::anyhow!(
                        "Caller for `{}` not found. The contract is registered in '{}' file, but not in the CLI builder. Make sure you have added it to the builder using `.contract::<{}>()`.",
                        deployed_contract.key_name(), path, deployed_contract.name()
                    )
                })?
                .clone();
            self.host_env.register_contract(
                deployed_contract.address(),
                deployed_contract.key_name(),
                caller
            );
        }
        Ok(())
    }

    /// Dispatches a parsed subcommand to its handler.
    ///
    /// `deploy` mutates the container; the other subcommands are read-only. An unknown verb returns
    /// an error (recoverable in the REPL) rather than panicking.
    fn dispatch(
        &self,
        cmd: &str,
        args: &ArgMatches,
        container: &mut DeployedContractsContainer
    ) -> Result<()> {
        match cmd {
            DEPLOY_SUBCOMMAND => self
                .deploy_cmd
                .as_ref()
                .ok_or_else(|| {
                    anyhow::anyhow!("Deploy command not found. Did you forget to add it?")
                })?
                .run(&self.host_env, args, &self.custom_types, container),
            CONTRACTS_SUBCOMMAND => self.run_command(&self.contracts_cmd, args, container),
            PRINT_EVENTS_SUBCOMMAND => self.run_command(&self.print_events_cmd, args, container),
            SCENARIOS_SUBCOMMAND => self.run_command(&self.scenarios_cmd, args, container),
            WHOAMI_SUBCOMMAND => self.run_command(&self.whoami_cmd, args, container),
            REPL_SUBCOMMAND => repl::run(self, container),
            _ => Err(anyhow::anyhow!("Unknown command: {cmd}"))
        }
    }

    fn run_command<T: OdraCommand>(
        &self,
        cmd: &T,
        args: &ArgMatches,
        container: &DeployedContractsContainer
    ) -> Result<()> {
        cmd.run(&self.host_env, args, &self.custom_types, container)
    }
}
