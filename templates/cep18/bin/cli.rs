//! This example demonstrates how to use the `odra-cli` tool to deploy and interact with a smart contract.

use {{project-name}}::token::{MyToken, MyTokenInitArgs};
use odra::host::HostEnv;
use odra::prelude::Address;
use odra::schema::casper_contract_schema::NamedCLType;
use odra_cli::{
    deploy::DeployScript,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer,
    DeployerExt, OdraCli,
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
                decimals: 10,
                initial_supply: 100_000_000_000u64.into(),
            },
            container,
            450_000_000_000, // Adjust gas limit as needed
        )?;

        Ok(())
    }
}

/// Scenario that transfers tokens to three specified recipients.
pub struct DistributeScenario;

impl Scenario for DistributeScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("r1", "Recipient 1 address", NamedCLType::Key).required(),
            CommandArg::new("r2", "Recipient 2 address", NamedCLType::Key),
            CommandArg::new("r3", "Recipient 3 address", NamedCLType::Key),
            CommandArg::new(
                "amount",
                "Amount to mint for each recipient",
                NamedCLType::U256,
            ).required(),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        let mut contract = container.contract_ref::<MyToken>(env)?;
        let recipient1 = args.get_single::<Address>("r1")?;
        let recipient2 = args.get_single::<Address>("r2");
        let recipient3 = args.get_single::<Address>("r3");
        let amount = args.get_single("amount").unwrap_or(1_000_000_000u64.into());

        env.set_gas(50_000_000);
        contract.transfer(&recipient1, &amount);
        assert!(contract.balance_of(&recipient1) == amount);

        match recipient2 {
            Ok(recipient2) => {
                env.set_gas(50_000_000);
                contract.transfer(&recipient2, &amount);
                assert!(contract.balance_of(&recipient2) == amount);
            }
            _ => println!("No second recipient provided, skipping transfer for recipient 2."),
        }
        match recipient3 {
            Ok(recipient3) => {
                env.set_gas(50_000_000);
                contract.transfer(&recipient3, &amount);
                assert_eq!(contract.balance_of(&recipient3), amount);
            }
            _ => println!("No third recipient provided, skipping transfer for recipient 3."),
        }

        Ok(())
    }
}

impl ScenarioMetadata for DistributeScenario {
    const NAME: &'static str = "distribute";
    const DESCRIPTION: &'static str =
        "Distributes tokens to three specified recipients by transferring a specified amount for each";
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for {{project-name}} smart contract")
        .deploy(MyTokenDeployScript)
        .contract::<MyToken>()
        .scenario(DistributeScenario)
        .build()
        .run();
}
