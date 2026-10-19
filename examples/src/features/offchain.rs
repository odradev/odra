//! This example shows `#[odra(offchain)]`: a function of the contract that is never deployed.
//!
//! It runs on the host (tests, scripts, odra-cli) against the contract's state, so it can be as
//! expensive as it likes: no gas, no transaction. The wasm and the schema do not contain it.
use odra::casper_types::U256;
use odra::prelude::*;

/// Keeps a balance per depositor and the list of depositors.
#[odra::module]
pub struct BalanceBook {
    balances: Mapping<Address, U256>,
    holders: List<Address>
}

#[odra::module]
impl BalanceBook {
    /// Credits the caller with `amount`.
    pub fn deposit(&mut self, amount: U256) {
        let caller = self.env().caller();
        if self.balances.get_or_default(&caller).is_zero() {
            self.holders.push(caller);
        }
        self.balances.add(&caller, amount);
    }

    /// The balance of `owner`.
    pub fn balance_of(&self, owner: &Address) -> U256 {
        self.balances.get_or_default(owner)
    }

    /// Every holder with its balance. A loop over the whole list is fine on the host and would
    /// be too expensive (or too big) as an entry point.
    #[odra(offchain)]
    pub fn all_balances(&self) -> Vec<(Address, U256)> {
        self.holders
            .iter()
            .map(|holder| (holder, self.balance_of(&holder)))
            .collect()
    }

    /// The balances of many owners in one call.
    #[odra(offchain)]
    pub fn balances_of(&self, owners: Vec<Address>) -> Vec<U256> {
        owners.iter().map(|owner| self.balance_of(owner)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::BalanceBook;
    use odra::casper_types::U256;
    use odra::host::{Deployer, NoArgs};
    use odra::prelude::*;
    use odra::schema::SchemaEntrypoints;

    #[test]
    fn offchain_functions_read_the_state_without_a_transaction() {
        let env = odra_test::env();
        let alice = env.get_account(1);
        let bob = env.get_account(2);
        let mut book = BalanceBook::deploy(&env, NoArgs);

        env.set_caller(alice);
        book.deposit(U256::from(10));
        env.set_caller(bob);
        book.deposit(U256::from(20));
        book.deposit(U256::from(5));

        assert_eq!(
            book.all_balances(),
            vec![(alice, U256::from(10)), (bob, U256::from(25))]
        );
        assert_eq!(
            book.balances_of(vec![bob, alice, env.get_account(3)]),
            vec![U256::from(25), U256::from(10), U256::zero()]
        );
        assert_eq!(book.try_balances_of(vec![]), Ok(vec![]));
    }

    #[test]
    fn offchain_functions_are_not_entry_points() {
        let names = |eps: Vec<odra::schema::casper_contract_schema::Entrypoint>| {
            eps.into_iter().map(|ep| ep.name).collect::<Vec<_>>()
        };
        assert_eq!(
            names(BalanceBook::schema_entrypoints()),
            vec!["deposit", "balance_of"]
        );
        assert_eq!(
            names(BalanceBook::schema_offchain_entrypoints()),
            vec!["all_balances", "balances_of"]
        );
    }
}
