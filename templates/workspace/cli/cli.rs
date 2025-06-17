//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use flapper::flapper::Flapper;
use flipper::flipper::Flipper;
use odra::host::{HostEnv, NoArgs};
use odra_cli::{
    deploy::DeployScript,
    scenario::{Scenario, ScenarioMetadata},
    CommandArg, DeployedContractsContainer, DeployerExt, OdraCli, ScenarioArgs, ScenarioError,
};

/// Deploys the `Flipper` and `Flapper` contracts.
pub struct ContractsDeployScript;

impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        let _ = Flipper::load_or_deploy(
            &env,
            NoArgs,
            container,
            250_000_000_000, // Adjust gas limit as needed
        )?;

        let _ = Flapper::load_or_deploy(
            &env,
            NoArgs,
            container,
            250_000_000_000, // Adjust gas limit as needed
        )?;

        Ok(())
    }
}

/// Scenario that flips the state of both `Flipper` and `Flapper` contracts.
pub struct FlipThemAll;

impl Scenario for FlipThemAll {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: DeployedContractsContainer,
        _args: ScenarioArgs,
    ) -> Result<(), ScenarioError> {
        let mut flipper = container.get_ref::<Flipper>(env)?;
        let mut flapper = container.get_ref::<Flapper>(env)?;

        env.set_gas(50_000_000);
        flipper.try_flip()?;
        flapper.try_flap()?;

        Ok(())
    }
}

impl ScenarioMetadata for FlipThemAll {
    const NAME: &'static str = "flip_them_all";
    const DESCRIPTION: &'static str = "Flips the state of the Flipper and Flapper contracts.";
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for {{project-name}} smart contract")
        .deploy(ContractsDeployScript)
        .contract::<Flipper>()
        .contract::<Flapper>()
        .scenario(FlipThemAll)
        .build()
        .run();
}
