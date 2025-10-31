//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.
use odra::host::HostEnv;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::DeployerExt;
use odra_cli::{
    deploy::DeployScript,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer, OdraCli
};
use odra_examples::features::storage::variable::{DogContract, DogContractInitArgs};
use std::vec;

/// Deploys the `DogContract` and adds it to the container.
pub struct DeployDogScript;
impl DeployScript for DeployDogScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer
    ) -> Result<(), odra_cli::deploy::Error> {
        _ = DogContract::load_or_deploy(
            env,
            DogContractInitArgs {
                barks: true,
                weight: 10,
                name: "Mantus".to_string()
            },
            container,
            350_000_000_000
        )?;

        Ok(())
    }
}

/// Checks if the name of the deployed dog matches the provided name.
pub struct DogCheckScenario;

impl Scenario for DogCheckScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![CommandArg::new("name", "The name of the dog", NamedCLType::String).required()]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args
    ) -> Result<(), Error> {
        let dog_contract = container.contract_ref::<DogContract>(env)?;
        let test_name = args.get_single::<String>("name")?;

        env.set_gas(50_000_000);
        let actual_name = dog_contract.try_name()?;

        assert_eq!(test_name, actual_name, "Dog name mismatch");

        Ok(())
    }
}

impl ScenarioMetadata for DogCheckScenario {
    const NAME: &'static str = "check";
    const DESCRIPTION: &'static str =
        "Checks if the name of the deployed dog matches the provided name";
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("Dog contract cli tool")
        .deploy(DeployDogScript)
        .contract::<DogContract>()
        .scenario::<DogCheckScenario>(DogCheckScenario)
        .build()
        .run();
}
