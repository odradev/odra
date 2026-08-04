//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use {{project-name}}::flipper::Flipper;
use odra::host::{HostEnv, NoArgs};
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    deploy::DeployScript,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer, DeployerExt,
    OdraCli, 
};

/// Deploys the `Flipper` and adds it to the container.
pub struct FlipperDeployScript;

impl DeployScript for FlipperDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer
    ) -> Result<(), odra_cli::deploy::Error> {
        let _flipper = Flipper::load_or_deploy(
            &env,
            NoArgs,
            container,
            350_000_000_000 // Adjust gas limit as needed
        )?;

        Ok(())
    }
}

/// Scenario that flips the state of the deployed `Flipper` contract a specified number of times.
pub struct FlippingScenario;

impl Scenario for FlippingScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![CommandArg::new(
            "number",
            "The number of times to flip the state",
            NamedCLType::U64,
        )]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args
    ) -> Result<(), Error> {
        let mut contract = container.contract_ref::<Flipper>(env)?;
        let n = args.get_single::<u64>("number")?;

        env.set_gas(5_000_000_000); // Adjust gas limit as needed
        for _ in 0..n {
            contract.try_flip()?;
        }

        Ok(())
    }
}

impl ScenarioMetadata for FlippingScenario {
    const NAME: &'static str = "flip";
    const DESCRIPTION: &'static str =
        "Flips the state of the deployed flipper contract a specified number of times";
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for {{project-name}} smart contract")
        .deploy(FlipperDeployScript)
        .contract::<Flipper>()
        .scenario(FlippingScenario)
        .build()
        .run();
}
