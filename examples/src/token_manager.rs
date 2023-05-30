use odra::{
    types::{Address, U256},
    Mapping, Variable
};

use crate::owned_token::OwnedToken;

#[odra::module]
pub struct TokenManager {
    tokens: Mapping<String, OwnedToken>,
    count: Variable<u32>
}

impl TokenManager {
    pub fn add_token(&mut self, name: String, decimals: u8, symbol: String) {
        self.tokens
            .get_instance(&name)
            .init(name, symbol, decimals, &U256::from(0));
        let new_count = self.count.get_or_default() + 1;
        self.count.set(&new_count);
    }
    pub fn balance_of(&self, token_name: String, owner: &Address) -> U256 {
        self.get_token(token_name).balance_of(owner)
    }
    pub fn mint(&mut self, token_name: String, account: &Address, amount: &U256) {
        self.get_token(token_name).mint(account, amount);
    }
    pub fn tokens_count(&self) -> u32 {
        self.count.get_or_default()
    }
    pub fn get_owner(&self, token_name: String) -> Address {
        self.get_token(token_name).get_owner()
    }
    pub fn set_owner(&mut self, token_name: String, new_owner: &Address) {
        self.get_token(token_name).change_ownership(new_owner);
    }
    fn get_token(&self, token_name: String) -> OwnedToken {
        self.tokens.get_instance(&token_name)
    }
}
pub struct TokenManagerDeployer;
#[cfg(feature = "mock-vm")]
impl TokenManagerDeployer {
    pub fn default() -> TokenManagerRef {
        use odra::types::CallArgs;
        use std::collections::HashMap;
        let mut entrypoints =
            HashMap::<String, (Vec<String>, fn(String, &CallArgs) -> Vec<u8>)>::new();
        entrypoints.insert(
            "add_token".to_string(),
            (
                {
                    let mut args: Vec<String> = Vec::new();
                    args.push("name".to_string());
                    args.push("decimals".to_string());
                    args.push("symbol".to_string());
                    args
                },
                |name, args| {
                    if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                        odra::contract_env::revert(odra::types::ExecutionError::non_payable());
                    }
                    let mut instance = <TokenManager as odra::Instance>::instance(name.as_str());
                    let result = instance.add_token(
                        &args.get("name"),
                        &args.get("decimals"),
                        &args.get("symbol")
                    );
                    Vec::new()
                }
            )
        );
        entrypoints.insert(
            "balance_of".to_string(),
            (
                {
                    let mut args: Vec<String> = Vec::new();
                    args.push("token_name".to_string());
                    args.push("owner".to_string());
                    args
                },
                |name, args| {
                    if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                        odra::contract_env::revert(odra::types::ExecutionError::non_payable());
                    }
                    let mut instance = <TokenManager as odra::Instance>::instance(name.as_str());
                    let result = instance.balance_of(&args.get("token_name"), &args.get("owner"));
                    odra::types::MockVMType::ser(&result).unwrap()
                }
            )
        );
        entrypoints.insert(
            "mint".to_string(),
            (
                {
                    let mut args: Vec<String> = Vec::new();
                    args.push("token_name".to_string());
                    args.push("account".to_string());
                    args.push("amount".to_string());
                    args
                },
                |name, args| {
                    if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                        odra::contract_env::revert(odra::types::ExecutionError::non_payable());
                    }
                    let mut instance = <TokenManager as odra::Instance>::instance(name.as_str());
                    let result = instance.mint(
                        &args.get("token_name"),
                        &args.get("account"),
                        &args.get("amount")
                    );
                    Vec::new()
                }
            )
        );
        entrypoints.insert(
            "tokens_count".to_string(),
            (
                {
                    let mut args: Vec<String> = Vec::new();
                    args
                },
                |name, args| {
                    if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                        odra::contract_env::revert(odra::types::ExecutionError::non_payable());
                    }
                    let mut instance = <TokenManager as odra::Instance>::instance(name.as_str());
                    let result = instance.tokens_count();
                    odra::types::MockVMType::ser(&result).unwrap()
                }
            )
        );
        entrypoints.insert(
            "get_owner".to_string(),
            (
                {
                    let mut args: Vec<String> = Vec::new();
                    args.push("token_name".to_string());
                    args
                },
                |name, args| {
                    if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                        odra::contract_env::revert(odra::types::ExecutionError::non_payable());
                    }
                    let mut instance = <TokenManager as odra::Instance>::instance(name.as_str());
                    let result = instance.get_owner(&args.get("token_name"));
                    odra::types::MockVMType::ser(&result).unwrap()
                }
            )
        );
        entrypoints.insert(
            "set_owner".to_string(),
            (
                {
                    let mut args: Vec<String> = Vec::new();
                    args.push("token_name".to_string());
                    args.push("new_owner".to_string());
                    args
                },
                |name, args| {
                    if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                        odra::contract_env::revert(odra::types::ExecutionError::non_payable());
                    }
                    let mut instance = <TokenManager as odra::Instance>::instance(name.as_str());
                    let result =
                        instance.set_owner(&args.get("token_name"), &args.get("new_owner"));
                    Vec::new()
                }
            )
        );
        let mut constructors =
            HashMap::<String, (Vec<String>, fn(String, &CallArgs) -> Vec<u8>)>::new();
        let address = odra::test_env::register_contract(None, constructors, entrypoints);
        TokenManagerRef::at(address)
    }
}
pub struct TokenManagerRef {
    address: odra::types::Address,
    attached_value: Option<odra::types::Balance>
}
#[automatically_derived]
impl ::core::clone::Clone for TokenManagerRef {
    #[inline]
    fn clone(&self) -> TokenManagerRef {
        TokenManagerRef {
            address: ::core::clone::Clone::clone(&self.address),
            attached_value: ::core::clone::Clone::clone(&self.attached_value)
        }
    }
}
impl TokenManagerRef {
    pub fn at(address: odra::types::Address) -> Self {
        Self {
            address,
            attached_value: None
        }
    }
    pub fn address(&self) -> odra::types::Address {
        self.address.clone()
    }
    pub fn with_tokens<T>(&self, amount: T) -> Self
    where
        T: Into<odra::types::Balance>
    {
        Self {
            address: self.address,
            attached_value: Some(amount.into())
        }
    }
}
impl TokenManagerRef {
    pub fn add_token(&mut self, name: String, decimals: u8, symbol: String) {
        let args = {
            let mut args = odra::types::CallArgs::new();
            args.insert("name", name.clone());
            args.insert("decimals", decimals.clone());
            args.insert("symbol", symbol.clone());
            args
        };
        odra::call_contract::<()>(self.address, "add_token", &args, self.attached_value);
    }
    pub fn balance_of(&self, token_name: String, owner: &Address) -> U256 {
        let args = {
            let mut args = odra::types::CallArgs::new();
            args.insert("token_name", token_name.clone());
            args.insert("owner", owner.clone());
            args
        };
        odra::call_contract(self.address, "balance_of", &args, self.attached_value)
    }
    pub fn mint(&mut self, token_name: String, account: &Address, amount: &U256) {
        let args = {
            let mut args = odra::types::CallArgs::new();
            args.insert("token_name", token_name.clone());
            args.insert("account", account.clone());
            args.insert("amount", amount.clone());
            args
        };
        odra::call_contract::<()>(self.address, "mint", &args, self.attached_value);
    }
    pub fn tokens_count(&self) -> u32 {
        let args = {
            let mut args = odra::types::CallArgs::new();
            args
        };
        odra::call_contract(self.address, "tokens_count", &args, self.attached_value)
    }
    pub fn get_owner(&self, token_name: String) -> Address {
        let args = {
            let mut args = odra::types::CallArgs::new();
            args.insert("token_name", token_name.clone());
            args
        };
        odra::call_contract(self.address, "get_owner", &args, self.attached_value)
    }
    pub fn set_owner(&mut self, token_name: String, new_owner: &Address) {
        let args = {
            let mut args = odra::types::CallArgs::new();
            args.insert("token_name", token_name.clone());
            args.insert("new_owner", new_owner.clone());
            args
        };
        odra::call_contract::<()>(self.address, "set_owner", &args, self.attached_value);
    }
}

// #[odra::module]
// impl TokenManager {
//     pub fn add_token(&mut self, name: String, decimals: u8, symbol: String) {
//         self.tokens
//             .get_instance(&name)
//             .init(name, symbol, decimals, &U256::from(0));

//         let new_count = self.count.get_or_default() + 1;
//         self.count.set(&new_count);
//     }

//     pub fn balance_of(&self, token_name: String, owner: &Address) -> U256 {
//         self.get_token(token_name).balance_of(owner)
//     }

//     pub fn mint(&mut self, token_name: String, account: &Address, amount: &U256) {
//         self.get_token(token_name).mint(account, amount);
//     }

//     pub fn tokens_count(&self) -> u32 {
//         self.count.get_or_default()
//     }

//     pub fn get_owner(&self, token_name: String) -> Address {
//         self.get_token(token_name).get_owner()
//     }

//     pub fn set_owner(&mut self, token_name: String, new_owner: &Address) {
//         self.get_token(token_name).change_ownership(new_owner);
//     }

//     fn get_token(&self, token_name: String) -> OwnedToken {
//         self.tokens.get_instance(&token_name)
//     }
// }

#[cfg(test)]
mod test {
    use super::{TokenManagerDeployer, TokenManagerRef};
    use odra::test_env::get_account;

    const PLS: &str = "PLS";
    const MCN: &str = "MCN";
    const PLASCOIN: &str = "Plascoin";
    const MY_COIN: &str = "MyCoin";
    const DECIMALS: u8 = 10;

    fn setup() -> TokenManagerRef {
        let mut contract = TokenManagerDeployer::default();

        contract.add_token(String::from(PLASCOIN), DECIMALS, String::from(PLS));
        contract.add_token(String::from(MY_COIN), DECIMALS, String::from(MCN));
        contract
    }

    #[test]
    fn minting_works() {
        let mut contract = setup();
        let (user1, user2, user3) = (get_account(0), get_account(1), get_account(2));
        let (pls_balance1, pls_balance2, pls_balance3) = (100.into(), 200.into(), 300.into());
        let (mcn_balance1, mcn_balance2, mcn_balance3) = (1000.into(), 2000.into(), 3000.into());

        let plascoin = String::from(PLASCOIN);
        let my_coin = String::from(MY_COIN);

        contract.mint(plascoin.clone(), &user1, &pls_balance1);
        contract.mint(plascoin.clone(), &user2, &pls_balance2);
        contract.mint(plascoin.clone(), &user3, &pls_balance3);

        contract.mint(my_coin.clone(), &user1, &mcn_balance1);
        contract.mint(my_coin.clone(), &user2, &mcn_balance2);
        contract.mint(my_coin.clone(), &user3, &mcn_balance3);

        assert_eq!(contract.balance_of(plascoin.clone(), &user1), pls_balance1);
        assert_eq!(contract.balance_of(plascoin.clone(), &user2), pls_balance2);
        assert_eq!(contract.balance_of(plascoin.clone(), &user3), pls_balance3);
        assert_eq!(contract.balance_of(my_coin.clone(), &user1), mcn_balance1);
        assert_eq!(contract.balance_of(my_coin.clone(), &user2), mcn_balance2);
        assert_eq!(contract.balance_of(my_coin.clone(), &user3), mcn_balance3);
    }

    #[test]
    fn tokens_count_works() {
        let contract = setup();
        assert_eq!(contract.tokens_count(), 2);
    }

    #[test]
    fn get_owner_works() {
        let mut contract = setup();
        let (owner, user2, user3) = (get_account(0), get_account(1), get_account(2));

        assert_eq!(contract.get_owner(String::from(PLASCOIN)), owner);
        assert_eq!(contract.get_owner(String::from(MY_COIN)), owner);

        contract.set_owner(String::from(PLASCOIN), &user2);
        contract.set_owner(String::from(MY_COIN), &user3);

        assert_eq!(contract.get_owner(String::from(PLASCOIN)), user2);
        assert_eq!(contract.get_owner(String::from(MY_COIN)), user3);
    }

    #[test]
    fn many_tokens_works() {
        let mut contract = TokenManagerDeployer::default();
        let (user, balance) = (get_account(0), 111.into());
        for i in 0..20 {
            contract.add_token(i.to_string(), DECIMALS, i.to_string());
            contract.mint(i.to_string(), &user, &balance);
            assert_eq!(contract.balance_of(i.to_string(), &user), balance);
        }
    }
}
