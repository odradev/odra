use std::collections::HashMap;

use anyhow::Result;
use clap::ArgMatches;
use odra::{
    contract_def::HasIdent,
    entry_point_callback::EntryPointsCaller,
    host::{EntryPointsCallerProvider, HostEnv},
    schema::{SchemaCustomTypes, SchemaEntrypoints, SchemaEvents},
    OdraContract
};

use crate::{
    cmd::{
        ContractsCmd, DeployCmd, DeployScript, MainCmd, MutableCommand, OdraCommand,
        PrintEventsCmd, Scenario, ScenarioMetadata, ScenariosCmd, CONTRACTS_SUBCOMMAND,
        DEPLOY_SUBCOMMAND, PRINT_EVENTS_SUBCOMMAND, SCENARIOS_SUBCOMMAND
    },
    container::{FileContractStorage, DEPLOYED_CONTRACTS_FILE},
    custom_types::CustomTypes,
    ContractProvider, DeployedContractsContainer
};

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
            let caller = self.callers.get(&name).unwrap_or_else(|| {
                let path = match &contracts_path {
                    Some(path) => path.to_str().map(|s| s.to_string()).unwrap_or_default(),
                    None => DEPLOYED_CONTRACTS_FILE.to_string()
                };
                prettycli::error(&format!(
                    "Caller for `{}` not found. The contract is registered in {:?} file, but not in the CLI builder. Make sure you have added it to the builder using `.contract::<{}>()`.",
                    &name, path, &name
                ));
                std::process::exit(1);
            }).clone();
            self.host_env.register_contract(address, name, caller);
        }

        let result = match cmd.as_str() {
            DEPLOY_SUBCOMMAND => self
                .deploy_cmd
                .as_ref()
                .unwrap_or_else(|| {
                    prettycli::error("Deploy command not found. Did you forget to add it?");
                    std::process::exit(1);
                })
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
