use crate::erc1155::erc1155_base::Erc1155Base;
use crate::erc1155::errors::Error;
use crate::erc1155::events::{TransferBatch, TransferSingle};
use crate::erc1155::Erc1155;
use odra::contract_env::{caller, revert};
use odra::types::event::OdraEvent;
use odra::types::Address;
use odra::types::Bytes;
use odra::types::U256;

use crate::extensions::ownable::{Ownable, OwnableExtension};

#[odra::module]
pub struct Erc1155Token {
    core: Erc1155Base,
    ownable: OwnableExtension
}

#[odra::module]
impl OwnedErc1155 for Erc1155Token {
    #[odra(init)]
    pub fn init(&mut self) {
        self.ownable.init(Some(caller()));
    }

    pub fn balance_of(&self, owner: Address, id: U256) -> U256 {
        self.core.balance_of(owner, id)
    }
    pub fn balance_of_batch(&self, owners: Vec<Address>, ids: Vec<U256>) -> Vec<U256> {
        self.core.balance_of_batch(owners, ids)
    }
    pub fn set_approval_for_all(&mut self, operator: Address, approved: bool) {
        self.core.set_approval_for_all(operator, approved);
    }
    pub fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        self.core.is_approved_for_all(owner, operator)
    }
    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        id: U256,
        amount: U256,
        data: Bytes
    ) {
        self.core.safe_transfer_from(from, to, id, amount, data);
    }

    pub fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Bytes
    ) {
        self.core
            .safe_batch_transfer_from(from, to, ids, amounts, data);
    }

    // Ownable
    pub fn renounce_ownership(&mut self) {
        self.ownable.renounce_ownership();
    }

    pub fn transfer_ownership(&mut self, new_owner: Option<Address>) {
        self.ownable.transfer_ownership(new_owner);
    }

    pub fn owner(&self) -> Option<Address> {
        self.ownable.owner()
    }

    pub fn mint(&mut self, to: Address, id: U256, amount: U256, data: Option<Bytes>) {
        let caller = caller();
        self.ownable.assert_owner(caller);

        let current_balance = self.core.balances.get(&(to, id)).unwrap_or_default();
        self.core.balances.set(&(to, id), amount + current_balance);

        TransferSingle {
            operator: Some(caller),
            from: None,
            to: Some(to),
            id,
            value: amount
        }
        .emit();

        self.core
            .safe_transfer_acceptance_check(caller, caller, to, id, amount, data);
    }

    pub fn mint_batch(
        &mut self,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Option<Bytes>
    ) {
        if ids.len() != amounts.len() {
            revert(Error::IdsAndAmountsLengthMismatch)
        }

        let caller = caller();
        self.ownable.assert_owner(caller);

        for (id, amount) in ids.iter().zip(amounts.iter()) {
            let current_balance = self.core.balances.get(&(to, *id)).unwrap_or_default();
            self.core
                .balances
                .set(&(to, *id), *amount + current_balance);
        }

        TransferBatch {
            operator: Some(caller),
            from: None,
            to: Some(to),
            ids: ids.clone(),
            values: amounts.clone()
        }
        .emit();

        self.core
            .safe_batch_transfer_acceptance_check(caller, caller, to, ids, amounts, data);
    }

    pub fn burn(&mut self, from: Address, id: U256, amount: U256) {
        let caller = caller();
        self.ownable.assert_owner(caller);

        let current_balance = self.core.balances.get(&(from, id)).unwrap_or_default();
        if current_balance < amount {
            revert(Error::InsufficientBalance)
        }

        self.core
            .balances
            .set(&(from, id), current_balance - amount);

        TransferSingle {
            operator: Some(caller),
            from: Some(from),
            to: None,
            id,
            value: amount
        }
        .emit();
    }

    pub fn burn_batch(&mut self, from: Address, ids: Vec<U256>, amounts: Vec<U256>) {
        if ids.len() != amounts.len() {
            revert(Error::IdsAndAmountsLengthMismatch)
        }

        let caller = caller();
        self.ownable.assert_owner(caller);

        for (id, amount) in ids.iter().zip(amounts.iter()) {
            let current_balance = self.core.balances.get(&(from, *id)).unwrap_or_default();
            if current_balance < *amount {
                revert(Error::InsufficientBalance)
            }
            self.core
                .balances
                .set(&(from, *id), current_balance - *amount);
        }

        TransferBatch {
            operator: Some(caller),
            from: Some(from),
            to: None,
            ids: ids.clone(),
            values: amounts.clone()
        }
        .emit();
    }
}

#[cfg(test)]
mod tests {
    use crate::erc1155::errors::Error;
    use crate::erc1155::events::{TransferBatch, TransferSingle};
    use crate::erc1155_receiver::Erc1155ReceiverRef;
    use crate::erc1155_token::{Erc1155Token, Erc1155TokenDeployer, Erc1155TokenRef};
    use odra::test_env::assert_exception;
    use odra::types::{Address, U256};
    use odra::{assert_events, test_env};

    // const NAME: &str = "PlascoinMultiNFT";
    // const SYMBOL: &str = "PLSMNFT";
    // const BASE_URI: &str = "https://plascoin.org/";

    struct TokenEnv {
        token: Erc1155TokenRef,
        alice: Address,
        bob: Address,
        carol: Address,
        owner: Address
    }

    fn setup() -> TokenEnv {
        TokenEnv {
            token: Erc1155TokenDeployer::init(),
            alice: test_env::get_account(1),
            bob: test_env::get_account(2),
            carol: test_env::get_account(3),
            owner: test_env::get_account(0)
        }
    }

    #[test]
    fn mint() {
        // Given a deployed contract
        let mut env = setup();

        // When we mint some tokens
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // Then the balance is updated
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 100.into());

        // And the event is emitted
        let contract = env.token;
        assert_events!(
            contract,
            TransferSingle {
                operator: Some(env.owner),
                from: None,
                to: Some(env.alice),
                id: U256::one(),
                value: 100.into()
            }
        );
    }

    #[test]
    fn mint_batch() {
        // Given a deployed contract
        let mut env = setup();

        // When we mint some tokens in batch
        env.token.mint_batch(
            env.alice,
            vec![U256::one(), U256::from(2)],
            vec![100.into(), 200.into()],
            None
        );

        // Then it emits the event
        let contract = Erc1155TokenRef::at(env.token.address());
        assert_events!(
            contract,
            TransferBatch {
                operator: Some(env.owner),
                from: None,
                to: Some(env.alice),
                ids: vec![U256::one(), U256::from(2)],
                values: vec![100.into(), 200.into()]
            }
        );

        // And the balances are updated
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 100.into());
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 200.into());
    }

    #[test]
    fn mint_batch_errors() {
        assert_exception(Error::IdsAndAmountsLengthMismatch, || {
            // Given a deployed contract
            let mut env = setup();
            // When we mint some tokens in batch with mismatching ids and amounts it errors out
            env.token.mint_batch(
                env.alice,
                vec![U256::one(), U256::from(2)],
                vec![100.into()],
                None
            );
        });
    }

    #[test]
    fn burn() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // When we burn some tokens
        env.token.burn(env.alice, U256::one(), 50.into());

        // Then the balance is updated
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());

        // And the event is emitted
        let contract = env.token;
        assert_events!(
            contract,
            TransferSingle {
                operator: Some(env.owner),
                from: Some(env.alice),
                to: None,
                id: U256::one(),
                value: 50.into()
            }
        );
    }

    #[test]
    fn burn_errors() {
        assert_exception(Error::InsufficientBalance, || {
            // Given a deployed contract
            let mut env = setup();

            // And some tokens minted
            env.token.mint(env.alice, U256::one(), 100.into(), None);

            // When we burn more tokens than we have it errors out
            env.token.burn(env.alice, U256::one(), 150.into());
        });

        assert_exception(Error::InsufficientBalance, || {
            // Given a deployed contract
            let mut env = setup();

            // When we burn non-existing tokens it errors out
            env.token.burn(env.alice, U256::one(), 150.into());
        });
    }

    #[test]
    fn burn_batch() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint_batch(
            env.alice,
            vec![U256::one(), U256::from(2)],
            vec![100.into(), 200.into()],
            None
        );

        // When we burn some tokens in batch
        env.token.burn_batch(
            env.alice,
            vec![U256::one(), U256::from(2)],
            vec![50.into(), 100.into()]
        );

        // Then the balances are updated
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 100.into());

        // And the event is emitted
        let contract = env.token;
        assert_events!(
            contract,
            TransferBatch {
                operator: Some(env.owner),
                from: Some(env.alice),
                to: None,
                ids: vec![U256::one(), U256::from(2)],
                values: vec![50.into(), 100.into()]
            }
        );
    }

    #[test]
    fn burn_batch_errors() {
        assert_exception(Error::IdsAndAmountsLengthMismatch, || {
            // Given a deployed contract
            let mut env = setup();

            // And some tokens minted
            env.token.mint_batch(
                env.alice,
                vec![U256::one(), U256::from(2)],
                vec![100.into(), 200.into()],
                None
            );

            // When we burn some tokens in batch with mismatching ids and amounts it errors out
            env.token
                .burn_batch(env.alice, vec![U256::one(), U256::from(2)], vec![50.into()]);
        });

        assert_exception(Error::InsufficientBalance, || {
            // Given a deployed contract
            let mut env = setup();

            // And some tokens minted
            env.token.mint_batch(
                env.alice,
                vec![U256::one(), U256::from(2)],
                vec![100.into(), 200.into()],
                None
            );

            // When we burn more tokens than we have it errors out
            env.token.burn_batch(
                env.alice,
                vec![U256::one(), U256::from(2)],
                vec![150.into(), 300.into()]
            );
        });

        assert_exception(Error::InsufficientBalance, || {
            // Given a deployed contract
            let mut env = setup();

            // When we burn non-existing tokens it errors out
            env.token.burn_batch(
                env.alice,
                vec![U256::one(), U256::from(2)],
                vec![150.into(), 300.into()]
            );
        });
    }
}
