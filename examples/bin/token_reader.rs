fn main() {
    use odra::types::Address;
    use odra_examples::erc20::Erc20Deployer;
    use std::str::FromStr;

    pub const NAME: &str = "Plascoin";
    pub const SYMBOL: &str = "PLS";
    pub const DECIMALS: u8 = 10;
    pub const INITIAL_SUPPLY: u32 = 10_000;
    
    let recipient = "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    let recipient = Address::from_str(recipient).unwrap();

    odra::client_env::set_gas(120_000_000_000u64);
    let mut token = Erc20Deployer::init(
        String::from(NAME),
        String::from(SYMBOL),
        DECIMALS,
        INITIAL_SUPPLY.into()
    );

    // assert_eq!(token.name(), NAME);

    // println!("The token name is {name}");

    odra::client_env::set_gas(5_000_000_000u64);
    token.transfer(recipient, 100.into());

    // let mut token = Erc20Deployer::register(address);

}

