//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use {{project-name}}::token::{MyToken, MyTokenInitArgs};
use odra::prelude::Address;
use odra::schema::casper_contract_schema::NamedCLType;
use odra::{casper_types::U256, host::HostEnv};
use odra_cli::{
    deploy::DeployScript,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer, DeployerExt,
    OdraCli,
};

/// Deploys the `MyToken` and adds it to the container.
pub struct MyTokenDeployScript;

impl DeployScript for MyTokenDeployScript {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        let _contract = MyToken::load_or_deploy(
            &env,
            MyTokenInitArgs {
                name: "MYToken".to_string(),
                symbol: "MT".to_string(),
            },
            container,
            450_000_000_000, // Adjust gas limit as needed
        )?;

        Ok(())
    }
}

/// Scenario that flips the state of the deployed `MyToken` contract a specified number of times.
pub struct MintAndBurn;

impl Scenario for MintAndBurn {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("to", "Token recipient", NamedCLType::Key).required(),
            CommandArg::new("token_id", "Token id", NamedCLType::U256).required(),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        let mut contract = container.contract_ref::<MyToken>(env)?;
        let to = args.get_single::<Address>("to")?;
        let token_id = args.get_single::<U256>("token_id")?;

        env.set_gas(50_000_000);
        contract.mint(to, token_id, vec![]);
        contract.burn(token_id);

        assert_eq!(
            contract.balance_of(to),
            U256::zero(),
            "Balance of recipient should be zero after minting and burning"
        );

        Ok(())
    }
}

impl ScenarioMetadata for MintAndBurn {
    const NAME: &'static str = "mint_and_burn";
    const DESCRIPTION: &'static str =
        "This scenario mints a token to a specified address and then burns it.";
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for {{project-name}} smart contract")
        .deploy(MyTokenDeployScript)
        .contract::<MyToken>()
        .scenario(MintAndBurn)
        .build()
        .run();
}
