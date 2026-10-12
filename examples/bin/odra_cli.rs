//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.
use odra::host::{Deployer, HostEnv, InstallConfig, NoArgs};
use odra::prelude::Addressable;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{cspr, DeployerExt};
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
        _ = DogContract::load_or_deploy_with_cfg(
            env,
            None,
            DogContractInitArgs {
                barks: true,
                weight: 10,
                name: "Mantus".to_string()
            },
            InstallConfig::upgradable::<DogContract>(),
            container,
            cspr!(450)
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

        let actual_name = dog_contract.try_name()?;
        if test_name != actual_name {
            odra_cli::log(format!("Dog name mismatch: expected {actual_name}"));
            return Err(Error::OdraError {
                message: "Error".to_string()
            });
        }

        Ok(())
    }
}

impl ScenarioMetadata for DogCheckScenario {
    const NAME: &'static str = "check";
    const DESCRIPTION: &'static str =
        "Checks if the name of the deployed dog matches the provided name";
}

/// Upgrades the deployed `DogContract` to the current wasm.
pub struct DogUpgradeScenario;

impl Scenario for DogUpgradeScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args
    ) -> Result<(), Error> {
        let dog_contract = container.contract_ref::<DogContract>(env)?;
        env.set_gas(cspr!(450));
        let upgraded = DogContract::try_upgrade(env, dog_contract.address(), NoArgs);
        env.set_gas(0);
        let upgraded = upgraded?;
        odra_cli::log(format!(
            "Upgraded DogContract, name after upgrade: {}",
            upgraded.try_name()?
        ));
        Ok(())
    }
}

impl ScenarioMetadata for DogUpgradeScenario {
    const NAME: &'static str = "upgrade";
    const DESCRIPTION: &'static str = "Upgrades the deployed DogContract to the current wasm";
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("Dog contract cli tool")
        .deploy(DeployDogScript)
        .contract::<DogContract>()
        .scenario::<DogCheckScenario>(DogCheckScenario)
        .scenario::<DogUpgradeScenario>(DogUpgradeScenario)
        .build()
        .run();
}
