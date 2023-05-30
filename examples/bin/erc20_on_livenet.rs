fn main() {
    use odra::client_env;
    use odra::types::Address;
    use odra_examples::erc20::Erc20Deployer;
    use std::str::FromStr;

    pub const NAME: &str = "Plascoin";
    pub const SYMBOL: &str = "PLS";
    pub const DECIMALS: u8 = 10;
    pub const INITIAL_SUPPLY: u32 = 10_000;

    let owner = client_env::caller();

    let recipient = "hash-2c4a6ce0da5d175e9638ec0830e01dd6cf5f4b1fbb0724f7d2d9de12b1e0f840";
    let recipient = Address::from_str(recipient).unwrap();

    client_env::set_gas(150_000_000_000u64);
    let mut token = Erc20Deployer::init(
        String::from(NAME),
        String::from(SYMBOL),
        DECIMALS,
        INITIAL_SUPPLY.into()
    );

    // Uncomment to use already deployed contract.
    // let address = "hash-3a20a5cd38e8826346faa673576657b03bbd4d86685e404baf9f1fa406a4f7a7";
    // let address = Address::from_str(address).unwrap();
    // let mut token = Erc20Deployer::register(address);

    let name = token.name();
    assert_eq!(name, NAME);

    client_env::set_gas(5_000_000_000u64);
    token.transfer(recipient, 100.into());

    println!("Owner's balance: {:?}", token.balance_of(owner));
    println!("Recipient's balance: {:?}", token.balance_of(recipient));
}
