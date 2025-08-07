//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use odra::host::HostEnv;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    deploy::DeployScript,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, DeployedContractsContainer, OdraCli, 
};

/// Deploys contracts and adds it to the container.
pub struct MyDeployScript;

impl DeployScript for MyDeployScript {
    fn deploy(
        &self,
        _env: &HostEnv,
        _container: &mut DeployedContractsContainer
    ) -> Result<(), odra_cli::deploy::Error> {

        Ok(())
    }
}

/// A custom scenario that demonstrates how to use the CLI tool with a custom argument.
pub struct MyScenario;

impl Scenario for MyScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![CommandArg::new(
            "my_arg",
            "A custom argument for the scenario",
            NamedCLType::String,
        )]
    }

    fn run(
        &self,
        _env: &HostEnv,
        _container: &DeployedContractsContainer,
        args: Args
    ) -> Result<(), Error> {
        // Read a contract reference from the container
        // let mut contract = container.contract_ref::<MyContract>(env)?;

        // Read the argument value
        let _my_arg = args.get_single::<String>("my_arg")?;

        Ok(())
    }
}

impl ScenarioMetadata for MyScenario {
    const NAME: &'static str = "my_scenario";
    const DESCRIPTION: &'static str = 
        "A custom scenario that demonstrates how to use the CLI tool with a custom argument.";
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for abc smart contract")
        .deploy(MyDeployScript)
        // .contract::<MyContract>()
        .scenario(MyScenario)
        .build()
        .run();
}
