fn main() {
    use odra::types::Address;
    use odra_examples::erc20::Erc20Deployer;
    use odra_examples::mapping::TokenManagerDeployer;
    use std::str::FromStr;

    pub const NAME: &str = "Plascoin";
    pub const SYMBOL: &str = "PLS";
    pub const DECIMALS: u8 = 10;
    pub const INITIAL_SUPPLY: u32 = 10_000;
    
    // odra::client_env::set_gas(108_000_000_000u64);
    // let mut token = TokenManagerDeployer::default();
    // token.
    // odra::client_env::set_gas(108_000_000_000_000u64);

    odra::client_env::set_gas(120_000_000_000u64);
    let mut token = Erc20Deployer::init(
        String::from(NAME),
        String::from(SYMBOL),
        DECIMALS,
        INITIAL_SUPPLY.into()
    );

    let address = "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    let address = Address::from_str(address).unwrap();
    let name = token.name();

    println!("The token name is {name}");

    odra::client_env::set_gas(1_000_000_000u64);
    token.transfer(address, 100.into());

    // let mut token = Erc20Deployer::register(address);

}

