use crate::cep18_token::Cep18ContractRef;
use odra::casper_types::U256;
use odra::prelude::*;
use odra::Address;

/// A client contract that can interact with CEP-18 contracts.
/// Only for purpose of testing CEP-18 module the same way as
/// the original CEP-18 module implementation.
#[odra::module]
pub struct Cep18ClientContract;

#[odra::module]
impl Cep18ClientContract {
    /// Calls total_supply method of the token contract at the given address.
    #[allow(dead_code)]
    pub fn check_total_supply(&self, address: Address) -> U256 {
        let token_contract = Cep18ContractRef::new(self.env(), address);
        token_contract.total_supply()
    }

    /// Calls balance_of method of the token contract at the given address.
    #[allow(dead_code)]
    pub fn check_balance_of(&self, address: Address, owner: Address) -> U256 {
        let token_contract = Cep18ContractRef::new(self.env(), address);
        token_contract.balance_of(&owner)
    }

    /// Calls allowance method of the token contract at the given address.
    #[allow(dead_code)]
    pub fn check_allowance_of(&self, address: Address, owner: Address, spender: Address) -> U256 {
        let token_contract = Cep18ContractRef::new(self.env(), address);
        token_contract.allowance(&owner, &spender)
    }

    /// Calls transfer method of the token contract at the given address.
    #[allow(dead_code)]
    pub fn transfer_as_stored_contract(&self, address: Address, recipient: Address, amount: U256) {
        let mut token_contract = Cep18ContractRef::new(self.env(), address);
        token_contract.transfer(&recipient, &amount)
    }

    /// Calls transfer_from method of the token contract at the given address.
    #[allow(dead_code)]
    pub fn transfer_from_as_stored_contract(
        &self,
        address: Address,
        owner: Address,
        recipient: Address,
        amount: U256
    ) {
        let mut token_contract = Cep18ContractRef::new(self.env(), address);
        token_contract.transfer_from(&owner, &recipient, &amount)
    }

    /// Calls approve method of the token contract at the given address.
    #[allow(dead_code)]
    pub fn approve_as_stored_contract(&self, address: Address, spender: Address, amount: U256) {
        let mut token_contract = Cep18ContractRef::new(self.env(), address);
        token_contract.approve(&spender, &amount)
    }
}
