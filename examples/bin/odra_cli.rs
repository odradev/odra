//! The Odra CLI of the examples: deploys the contracts and runs the livenet scenarios.
//!
//! Every scenario that used to be a separate `*_on_livenet` binary lives here:
//!
//! ```bash
//! cargo run --bin odra_cli --features livenet -- deploy
//! cargo run --bin odra_cli -- contract DogContract name
//! cargo run --bin odra_cli -- scenario erc20-transfer --amount 1000
//! cargo run --bin odra_cli -- --help
//! ```
use core::time::Duration;
use odra::casper_types::bytesrepr::Bytes;
use odra::casper_types::{U256, U512};
use odra::host::{Deployer, HostEnv, HostRef, HostRefLoader, InstallConfig, NoArgs};
use odra::prelude::*;
use odra::schema::casper_contract_schema::NamedCLType;
use odra::schema::NamedCLTyped;
use odra_cli::{
    cspr,
    deploy::DeployScript,
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg, ContractProvider, DeployedContractsContainer, DeployerExt, OdraCli
};
use odra_examples::contracts::gasless_cep18::authorization::{
    sign_transfer_authorization, TransferWithAuthorization
};
use odra_examples::contracts::gasless_cep18::{GaslessCep18, GaslessCep18InitArgs};
use odra_examples::contracts::tlw::{TimeLockWallet, TimeLockWalletInitArgs};
use odra_examples::factory::counter::{
    BetterCounterFactory, BetterCounterUpgradeArgs, Counter, CounterFactory
};
use odra_examples::features::storage::variable::{DogContract, DogContractInitArgs};
use odra_modules::cep18_token::{Cep18, Cep18InitArgs};
use odra_modules::erc20::{Erc20, Erc20InitArgs};

/// Deploys every contract the scenarios need, reusing the ones already in the container.
pub struct DeployScriptForExamples;

impl DeployScript for DeployScriptForExamples {
    fn deploy(
        &self,
        env: &HostEnv,
        container: &mut DeployedContractsContainer
    ) -> Result<(), odra_cli::deploy::Error> {
        DogContract::load_or_deploy(
            env,
            DogContractInitArgs {
                barks: true,
                weight: 10,
                name: "Mantus".to_string()
            },
            container,
            cspr!(350)
        )?;
        Erc20::load_or_deploy(env, erc20_args(), container, cspr!(450))?;
        Cep18::load_or_deploy(
            env,
            Cep18InitArgs {
                name: "Plascoin".to_string(),
                symbol: "PLS".to_string(),
                decimals: 2,
                initial_supply: U256::from(10_000)
            },
            container,
            cspr!(300)
        )?;
        // A one-second lock, so the `tlw` scenario can withdraw right after depositing.
        TimeLockWallet::load_or_deploy(
            env,
            TimeLockWalletInitArgs {
                lock_duration: Duration::from_secs(1).as_millis() as u64
            },
            container,
            cspr!(300)
        )?;
        Ok(())
    }
}

fn erc20_args() -> Erc20InitArgs {
    Erc20InitArgs {
        name: "Plascoin".to_string(),
        symbol: "PLS".to_string(),
        decimals: 10,
        initial_supply: Some(U256::from(10_000))
    }
}

/// `--recipient` if given, otherwise the second configured account.
fn recipient_or_second_account(env: &HostEnv, args: &Args) -> Address {
    args.get_single::<Address>("recipient")
        .unwrap_or_else(|_| env.get_account(1))
}

fn recipient_arg() -> CommandArg {
    CommandArg::new(
        "recipient",
        "Recipient of the tokens; defaults to the second configured account",
        Address::ty()
    )
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

/// Transfers ERC20 tokens and prints both balances (formerly `erc20_on_livenet`).
pub struct Erc20TransferScenario;

impl Scenario for Erc20TransferScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            recipient_arg(),
            CommandArg::new("amount", "Amount of tokens", NamedCLType::U256).required(),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args
    ) -> Result<(), Error> {
        let mut token = container.contract_ref::<Erc20>(env)?;
        let owner = env.caller();
        let recipient = recipient_or_second_account(env, &args);
        let amount = args.get_single::<U256>("amount")?;

        odra_cli::log(format!("Token name: {}", token.try_name()?));
        env.set_gas(cspr!(3));
        token.try_transfer(&recipient, &amount)?;
        odra_cli::log(format!(
            "Owner's balance: {}",
            token.try_balance_of(&owner)?
        ));
        odra_cli::log(format!(
            "Recipient's balance: {}",
            token.try_balance_of(&recipient)?
        ));
        Ok(())
    }
}

impl ScenarioMetadata for Erc20TransferScenario {
    const NAME: &'static str = "erc20-transfer";
    const DESCRIPTION: &'static str =
        "Transfers ERC20 tokens to an account and prints the balances";
}

/// Transfers and approves CEP-18 tokens (formerly `cep18_on_livenet`).
pub struct Cep18TransferScenario;

impl Scenario for Cep18TransferScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            recipient_arg(),
            CommandArg::new("amount", "Amount of tokens", NamedCLType::U256).required(),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args
    ) -> Result<(), Error> {
        let mut token = container.contract_ref::<Cep18>(env)?;
        let owner = env.caller();
        let recipient = recipient_or_second_account(env, &args);
        let amount = args.get_single::<U256>("amount")?;

        odra_cli::log(format!("Token name: {}", token.try_name()?));
        env.set_gas(cspr!(3));
        token.try_transfer(&recipient, &amount)?;
        token.try_approve(&recipient, &(amount * 5))?;
        odra_cli::log(format!(
            "Owner's balance: {}",
            token.try_balance_of(&owner)?
        ));
        odra_cli::log(format!(
            "Recipient's balance: {}",
            token.try_balance_of(&recipient)?
        ));
        odra_cli::log(format!(
            "Recipient's allowance: {}",
            token.try_allowance(&owner, &recipient)?
        ));
        Ok(())
    }
}

impl ScenarioMetadata for Cep18TransferScenario {
    const NAME: &'static str = "cep18-transfer";
    const DESCRIPTION: &'static str =
        "Transfers CEP-18 tokens to an account, approves five times as much and prints the balances";
}

/// Deposits CSPR into the time lock wallet and withdraws most of it (formerly `tlw_on_livenet`).
pub struct TimeLockWalletScenario;

impl Scenario for TimeLockWalletScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        _args: Args
    ) -> Result<(), Error> {
        let mut wallet = container.contract_ref::<TimeLockWallet>(env)?;
        let caller = env.caller();

        env.set_gas(cspr!(4));
        wallet.with_tokens(U512::from(cspr!(100))).try_deposit()?;
        odra_cli::log(format!(
            "Balance after deposit: {}",
            wallet.try_get_balance(&caller)?
        ));
        wallet.try_withdraw(&U512::from(cspr!(99)))?;
        odra_cli::log(format!(
            "Balance after withdrawal: {}",
            wallet.try_get_balance(&caller)?
        ));
        Ok(())
    }
}

impl ScenarioMetadata for TimeLockWalletScenario {
    const NAME: &'static str = "tlw";
    const DESCRIPTION: &'static str = "Deposits 100 CSPR into the TimeLockWallet and withdraws 99";
}

/// Shows what `InstallConfig::allow_key_override` does (formerly `odra_cfg_on_livenet`): the same
/// package key can be installed twice only when overriding is allowed.
pub struct InstallConfigScenario;

impl Scenario for InstallConfigScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        _container: &DeployedContractsContainer,
        _args: Args
    ) -> Result<(), Error> {
        let cfg = |allow_key_override| InstallConfig {
            package_named_key: "OverrideDemo".to_string(),
            is_upgradable: false,
            allow_key_override
        };

        env.set_gas(cspr!(450));
        let first = Erc20::try_deploy_with_cfg(env, erc20_args(), cfg(false))?;
        odra_cli::log(format!("First install: {}", first.address().to_string()));

        env.set_gas(cspr!(450));
        let second = Erc20::try_deploy_with_cfg(env, erc20_args(), cfg(true))?;
        odra_cli::log(format!(
            "Second install with allow_key_override = true: {}",
            second.address().to_string()
        ));

        env.set_gas(cspr!(450));
        let third = Erc20::try_deploy_with_cfg(env, erc20_args(), cfg(false));
        odra_cli::log(format!(
            "Third install with allow_key_override = false: {:?}",
            third.map(|t| t.address().to_string())
        ));
        Ok(())
    }
}

impl ScenarioMetadata for InstallConfigScenario {
    const NAME: &'static str = "install-config";
    const DESCRIPTION: &'static str =
        "Installs the same package key three times to show allow_key_override";
}

/// Deploys a factory, spawns child contracts, upgrades the factory and the children (formerly
/// `factory_example`).
pub struct FactoryScenario;

impl Scenario for FactoryScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        _container: &DeployedContractsContainer,
        _args: Args
    ) -> Result<(), Error> {
        env.set_gas(cspr!(480));
        let mut factory = CounterFactory::try_deploy_with_cfg(
            env,
            NoArgs,
            InstallConfig {
                package_named_key: "TestFactory".to_string(),
                is_upgradable: true,
                allow_key_override: true
            }
        )?;

        env.set_gas(cspr!(270));
        let (from_ten, _) = factory.try_new_contract(String::from("FromTen"), 10)?;
        let (from_two, _) = factory.try_new_contract(String::from("FromTwo"), 2)?;
        odra_cli::log(format!(
            "Children: FromTen at {}, FromTwo at {}",
            from_ten.to_string(),
            from_two.to_string()
        ));

        env.set_gas(cspr!(500));
        let mut new_factory = BetterCounterFactory::try_upgrade(env, factory.address(), NoArgs)?;
        new_factory.try_upgrade_child_contract(String::from("FromTen"), 122)?;
        odra_cli::log(format!(
            "FromTen after upgrade: {}",
            Counter::load(env, from_ten).try_value()?
        ));

        env.set_gas(cspr!(900));
        let args = [("FromTwo", 42u32)]
            .into_iter()
            .map(|(name, new_value)| (name.to_string(), BetterCounterUpgradeArgs { new_value }))
            .collect::<BTreeMap<_, _>>();
        new_factory.try_batch_upgrade_child_contract(args)?;
        odra_cli::log(format!(
            "FromTwo after batch upgrade: {}",
            Counter::load(env, from_two).try_value()?
        ));
        Ok(())
    }
}

impl ScenarioMetadata for FactoryScenario {
    const NAME: &'static str = "factory";
    const DESCRIPTION: &'static str =
        "Deploys a counter factory, spawns children, upgrades the factory and the children";
}

/// Main function to run the CLI tool.
pub fn main() {
    let cli = OdraCli::new()
        .about("Odra examples cli tool")
        .deploy(DeployScriptForExamples)
        .contract::<DogContract>()
        .contract::<Erc20>()
        .contract::<Cep18>()
        .contract::<TimeLockWallet>()
        .scenario(DogCheckScenario)
        .scenario(Erc20TransferScenario)
        .scenario(Cep18TransferScenario)
        .scenario(TimeLockWalletScenario)
        .scenario(InstallConfigScenario)
        .scenario(FactoryScenario)
        .scenario(GaslessTransferScenario);
    cli.build().run();
}

/// A gasless CEP-18 transfer authorized with an EIP-712 signature (formerly `gassless_example`):
/// Alice signs a transfer, Charlie submits it and pays the gas.
pub struct GaslessTransferScenario;

impl Scenario for GaslessTransferScenario {
    fn args(&self) -> Vec<CommandArg> {
        vec![]
    }

    fn run(
        &self,
        env: &HostEnv,
        _container: &DeployedContractsContainer,
        _args: Args
    ) -> Result<(), Error> {
        let chain_name = std::env::var("ODRA_CASPER_LIVENET_CHAIN_NAME")
            .unwrap_or_else(|_| "casper-test".to_string());
        env.set_gas(cspr!(500));
        let mut contract = GaslessCep18::try_deploy(
            env,
            GaslessCep18InitArgs {
                chain_name: chain_name.clone()
            }
        )?;
        let (alice, bob, charlie) = (env.get_account(0), env.get_account(1), env.get_account(2));
        let auth = TransferWithAuthorization {
            from: alice,
            to: bob,
            value: U256::from(1000),
            valid_after: 0,
            valid_before: u64::MAX,
            nonce: [0u8; 32]
        };
        let signature = sign_transfer_authorization(env, &chain_name, &contract.address(), &auth);
        odra_cli::log(format!(
            "Signature: 0x{}",
            signature
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        ));

        env.set_caller(charlie);
        contract.try_transfer_with_authorization(
            alice,
            bob,
            auth.value,
            auth.valid_after,
            auth.valid_before,
            Bytes::from(auth.nonce.to_vec()),
            env.public_key(&alice),
            signature
        )?;
        odra_cli::log(format!(
            "Transfer with authorization succeeded; Bob's balance: {}",
            contract.try_balance_of(&bob)?
        ));
        Ok(())
    }
}

impl ScenarioMetadata for GaslessTransferScenario {
    const NAME: &'static str = "gasless";
    const DESCRIPTION: &'static str =
        "Submits a CEP-18 transfer signed by another account (EIP-712 authorization)";
}
