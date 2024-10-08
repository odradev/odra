//! Deploys an ERC20 contract and transfers some tokens to another address.
use odra::casper_types::U256;
use odra::host::{Deployer, HostEnv, HostRef, HostRefLoader, OdraConfig};
use odra::Address;
use odra_modules::cep78::token;
use odra_modules::erc20::{Erc20, Erc20HostRef, Erc20InitArgs};
use std::str::FromStr;

struct Cfg {
    is_upgradable: bool,
    allow_key_override: bool
}

impl OdraConfig for Cfg {
    fn package_hash(&self) -> String {
        "aaerc20".to_string()
    }

    fn is_upgradable(&self) -> bool {
        self.is_upgradable
    }

    fn allow_key_override(&self) -> bool {
        self.allow_key_override
    }
}

fn main() {
    let env = odra_casper_livenet_env::env();

    env.set_gas(100_000_000_000u64);
    let result = Erc20::try_deploy_with_cfg(
        &env,
        erc20_args(),
        Cfg {
            is_upgradable: false,
            allow_key_override: false
        }
    );

    // println!("Deploy result: {:?}", result.err());
    let token = result.unwrap();
    println!("Token address: {:?}", token.address());
    println!("Token name: {}", token.name());

    env.set_gas(100_000_000_000u64);
    let result = Erc20::try_deploy_with_cfg(
        &env,
        erc20_args(),
        Cfg {
            is_upgradable: false,
            allow_key_override: true
        }
    );

    // println!("Deploy result: {:?}", result.err());
    let token = result.unwrap();
    println!("Token address: {:?}", token.address());
    println!("Token name: {}", token.name());

    env.set_gas(100_000_000_000u64);
    let result = Erc20::try_deploy_with_cfg(
        &env,
        erc20_args(),
        Cfg {
            is_upgradable: false,
            allow_key_override: false
        }
    );

    println!("Deploy result: {:?}", result.err());
}

fn erc20_args() -> Erc20InitArgs {
    let name = String::from("Plascoin");
    let symbol = String::from("PLS");
    let decimals = 10u8;
    let initial_supply = Some(U256::from(10_000));

    Erc20InitArgs {
        name,
        symbol,
        decimals,
        initial_supply
    }
}
