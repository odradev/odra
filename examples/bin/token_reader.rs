fn main() {
    use odra::types::Address;
    use odra_examples::erc20::Erc20Deployer;
    use std::str::FromStr;

    let address = "hash-57cbf48566ee3f59b2ae8ef45d28a49a462899f9765c2c15921b4ac5197a2f4d";
    let address = Address::from_str(address).unwrap();
    let token = Erc20Deployer::register(address);
    let name = token.name();

    println!("The token name is {name}");
}
