fn main() {
    use odra::types::Address;
    use odra_examples::erc20::Erc20Deployer;
    use std::str::FromStr;

    pub const NAME: &str = "Plascoin";
    pub const SYMBOL: &str = "PLS";
    pub const DECIMALS: u8 = 10;
    pub const INITIAL_SUPPLY: u32 = 10_000;
    
    let owner = "account-hash-3b4ffcfb21411ced5fc1560c3f6ffed86f4885e5ea05cde49d90962a48a14d95";
    let owner = Address::from_str(owner).unwrap();

    let recipient = "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    let recipient = Address::from_str(recipient).unwrap();

    // odra::client_env::set_gas(150_000_000_000u64);
    // let mut token = Erc20Deployer::init(
    //     String::from(NAME),
    //     String::from(SYMBOL),
    //     DECIMALS,
    //     INITIAL_SUPPLY.into()
    // );

    let token_address = Address::from_str("hash-6373d5f747595037069bdca1e972ab588142dd65168747f1bc920b976c0134f3").unwrap();
    let mut token = Erc20Deployer::register(token_address);

    let name = token.name();
    assert_eq!(token.name(), NAME);

    for i in 0..10 {
        odra::client_env::set_gas(5_000_000_000u64);
        token.transfer(recipient, 100.into());
        println!("owner balance: {:?}", token.balance_of(owner));
        println!("recipient balance: {:?}", token.balance_of(recipient));
    }
}

