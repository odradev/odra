//! `self.contract_address()` in a `#[odra::ref_helpers]` helper works on the `ContractRef` and the
//! `HostRef` copy with nothing but the prelude imported, and stays unambiguous when `ContractRef`
//! and `HostRef` are imported as well.
#![deny(warnings)]

use odra::casper_types::U256;
use odra::prelude::*;

#[odra::module]
pub struct Vault {
    balances: Mapping<Address, U256>
}

#[odra::module]
impl Vault {
    pub fn balance_of(&self, owner: &Address) -> U256 {
        self.balances.get_or_default(owner)
    }
}

#[odra::ref_helpers]
impl Vault {
    pub fn own_address(&self) -> Address {
        self.contract_address()
    }

    pub fn own_balance(&self) -> U256 {
        self.balance_of(&self.contract_address())
    }
}

#[odra::external_contract]
pub trait Token {
    fn balance_of(&self, owner: &Address) -> U256;
}

#[odra::ref_helpers]
impl Token {
    pub fn own_balance(&self) -> U256 {
        self.balance_of(&self.contract_address())
    }
}

#[odra::module]
pub struct Reader {
    token: External<TokenContractRef>
}

#[odra::module]
impl Reader {
    pub fn token_own_balance(&self) -> U256 {
        self.token.own_balance()
    }

    pub fn vault_address(&self, vault: Address) -> Address {
        <VaultContractRef as odra::ContractRef>::new(self.env(), vault).own_address()
    }
}

mod with_trait_imports {
    use super::*;
    use odra::host::{HostEnv, HostRef};
    use odra::ContractRef;

    #[allow(dead_code)]
    fn contract_side(env: Rc<ContractEnv>, address: Address) -> (Address, Address, U256) {
        let vault = VaultContractRef::new(env, address);
        (*vault.address(), vault.contract_address(), vault.own_balance())
    }

    #[allow(dead_code)]
    fn host_side(env: &HostEnv, address: Address) -> (Address, Address, U256) {
        let token = TokenHostRef::new(address, env.clone());
        (token.address(), token.contract_address(), token.own_balance())
    }
}

fn main() {}
