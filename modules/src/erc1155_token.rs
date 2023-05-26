use crate::access::Ownable;
use crate::erc1155::erc1155_base::Erc1155Base;
use crate::erc1155::errors::Error;
use crate::erc1155::events::{TransferBatch, TransferSingle};
use crate::erc1155::Erc1155;
use odra::contract_env::{caller, revert};
use odra::types::event::OdraEvent;
use odra::types::Address;
use odra::types::Bytes;
use odra::types::U256;

/// The ERC1155 token implementation.
/// It uses the ERC1155 base implementation and the Ownable module.
#[odra::module(events = [TransferBatch, TransferSingle])]
pub struct Erc1155Token {
    core: Erc1155Base,
    ownable: Ownable
}

#[odra::module]
impl OwnedErc1155 for Erc1155Token {
    #[odra(init)]
    pub fn init(&mut self) {
        self.ownable.init();
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
        data: Option<Bytes>
    ) {
        self.core.safe_transfer_from(from, to, id, amount, data);
    }

    pub fn safe_batch_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
        data: Option<Bytes>
    ) {
        self.core
            .safe_batch_transfer_from(from, to, ids, amounts, data);
    }

    // Ownable
    pub fn renounce_ownership(&mut self) {
        self.ownable.renounce_ownership();
    }

    pub fn transfer_ownership(&mut self, new_owner: Address) {
        self.ownable.transfer_ownership(new_owner);
    }

    pub fn owner(&self) -> Address {
        self.ownable.get_owner()
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
            ids,
            values: amounts
        }
        .emit();
    }
}

#[cfg(test)]
mod tests {
    use crate::erc1155::errors::Error;
    use crate::erc1155::events::{ApprovalForAll, TransferBatch, TransferSingle};
    use crate::erc1155_receiver::events::{BatchReceived, SingleReceived};
    use crate::erc1155_receiver::Erc1155ReceiverDeployer;
    use crate::erc1155_token::{Erc1155TokenDeployer, Erc1155TokenRef};
    use crate::wrapped_native::WrappedNativeTokenDeployer;
    use odra::test_env::assert_exception;
    use odra::types::VmError::NoSuchMethod;
    use odra::types::{Address, Bytes, OdraError, U256};
    use odra::{assert_events, test_env};

    struct TokenEnv {
        token: Erc1155TokenRef,
        alice: Address,
        bob: Address,
        owner: Address
    }

    fn setup() -> TokenEnv {
        TokenEnv {
            token: Erc1155TokenDeployer::init(),
            alice: test_env::get_account(1),
            bob: test_env::get_account(2),
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
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        assert_exception(Error::InsufficientBalance, || {
            // When we burn more tokens than we have it errors out
            env.token.burn(env.alice, U256::one(), 150.into());
        });

        // Given a deployed contract
        let mut env = setup();
        assert_exception(Error::InsufficientBalance, || {
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

    #[test]
    fn balance_of() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 200.into(), None);
        env.token.mint(env.bob, U256::one(), 300.into(), None);

        // Then it returns the correct balance
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 100.into());
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 200.into());
        assert_eq!(env.token.balance_of(env.bob, U256::one()), 300.into());
        assert_eq!(env.token.balance_of(env.bob, U256::from(2)), 0.into());
    }

    #[test]
    fn balance_of_batch() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint_batch(
            env.alice,
            vec![U256::one(), U256::from(2)],
            vec![100.into(), 200.into()],
            None
        );
        env.token.mint_batch(
            env.bob,
            vec![U256::one(), U256::from(2)],
            vec![300.into(), 400.into()],
            None
        );

        // Then it returns the correct balances
        assert_eq!(
            env.token.balance_of_batch(
                vec![env.alice, env.alice, env.alice, env.bob, env.bob, env.bob],
                vec![
                    U256::one(),
                    U256::from(2),
                    U256::from(3),
                    U256::one(),
                    U256::from(2),
                    U256::from(3)
                ]
            ),
            // TODO: Why it gives deserialization error when mismatched?
            vec![
                U256::from(100),
                U256::from(200),
                U256::zero(),
                U256::from(300),
                U256::from(400),
                U256::zero()
            ]
        );
    }

    #[test]
    fn balance_of_batch_errors() {
        assert_exception(Error::AccountsAndIdsLengthMismatch, || {
            // Given a deployed contract
            let mut env = setup();

            // And some tokens minted
            env.token.mint_batch(
                env.alice,
                vec![U256::one(), U256::from(2)],
                vec![100.into(), 200.into()],
                None
            );

            // When we query balances with mismatching ids and addresses it errors out
            env.token.balance_of_batch(
                vec![env.alice, env.alice, env.alice],
                vec![U256::one(), U256::from(2)]
            );
        });
    }

    #[test]
    fn set_approval_for_all() {
        // Given a deployed contract
        let mut env = setup();

        // When we set approval for all
        test_env::set_caller(env.alice);
        env.token.set_approval_for_all(env.bob, true);

        // Then the approval is set
        assert!(env.token.is_approved_for_all(env.alice, env.bob));

        // And the event is emitted
        let contract = env.token;
        assert_events!(
            contract,
            ApprovalForAll {
                owner: env.alice,
                operator: env.bob,
                approved: true
            }
        );
    }

    #[test]
    fn unset_approval_for_all() {
        // Given a deployed contract
        let mut env = setup();

        // And approval for all set
        test_env::set_caller(env.alice);
        env.token.set_approval_for_all(env.bob, true);

        // When we unset approval for all
        test_env::set_caller(env.alice);
        env.token.set_approval_for_all(env.bob, false);

        // Then the approval is unset
        assert!(!env.token.is_approved_for_all(env.alice, env.bob));

        // And the event is emitted
        let contract = env.token;
        assert_events!(
            contract,
            ApprovalForAll {
                owner: env.alice,
                operator: env.bob,
                approved: false
            }
        );
    }

    #[test]
    fn set_approval_to_self() {
        assert_exception(Error::ApprovalForSelf, || {
            // Given a deployed contract
            let mut env = setup();

            // Then approving for self throws an error
            test_env::set_caller(env.alice);
            env.token.set_approval_for_all(env.alice, true);
        });
    }

    #[test]
    fn safe_transfer_from() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // When we transfer tokens
        test_env::set_caller(env.alice);
        env.token
            .safe_transfer_from(env.alice, env.bob, U256::one(), 50.into(), None);

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.bob, U256::one()), 50.into());

        // And the event is emitted
        let contract = env.token;
        assert_events!(
            contract,
            TransferSingle {
                operator: Some(env.alice),
                from: Some(env.alice),
                to: Some(env.bob),
                id: U256::one(),
                value: 50.into()
            }
        );
    }

    #[test]
    fn safe_transfer_from_approved() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // And approval for all set
        test_env::set_caller(env.alice);
        env.token.set_approval_for_all(env.bob, true);

        // When we transfer tokens
        test_env::set_caller(env.bob);
        env.token
            .safe_transfer_from(env.alice, env.bob, U256::one(), 50.into(), None);

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.bob, U256::one()), 50.into());

        // And the event is emitted
        let contract = env.token;
        assert_events!(
            contract,
            TransferSingle {
                operator: Some(env.bob),
                from: Some(env.alice),
                to: Some(env.bob),
                id: U256::one(),
                value: 50.into()
            }
        );
    }

    #[test]
    fn safe_transfer_from_errors() {
        assert_exception(Error::InsufficientBalance, || {
            // Given a deployed contract
            let mut env = setup();

            // And some tokens minted
            env.token.mint(env.alice, U256::one(), 100.into(), None);

            // When we transfer more tokens than we have it errors out
            test_env::set_caller(env.alice);
            env.token
                .safe_transfer_from(env.alice, env.bob, U256::one(), 200.into(), None);
        });

        assert_exception(Error::NotAnOwnerOrApproved, || {
            // Given a deployed contract
            // test_env::set_caller(test_env::get_account(0));
            let mut env = setup();
            // And some tokens minted
            env.token.mint(env.alice, U256::one(), 100.into(), None);

            // When we transfer not our tokens it errors out
            test_env::set_caller(env.bob);
            env.token
                .safe_transfer_from(env.alice, env.bob, U256::one(), 100.into(), None);
        });
    }

    #[test]
    fn safe_batch_transfer_from() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 200.into(), None);

        // When we transfer tokens
        test_env::set_caller(env.alice);
        env.token.safe_batch_transfer_from(
            env.alice,
            env.bob,
            vec![U256::one(), U256::from(2)],
            vec![50.into(), 100.into()],
            None
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 100.into());
        assert_eq!(env.token.balance_of(env.bob, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.bob, U256::from(2)), 100.into());

        // And the event is emitted
        let contract = env.token;
        assert_events!(
            contract,
            TransferBatch {
                operator: Some(env.alice),
                from: Some(env.alice),
                to: Some(env.bob),
                ids: vec![U256::one(), U256::from(2)],
                values: vec![50.into(), 100.into()]
            }
        );
    }

    #[test]
    fn safe_batch_transfer_from_approved() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 200.into(), None);

        // And approval for all set
        test_env::set_caller(env.alice);
        env.token.set_approval_for_all(env.bob, true);

        // When we transfer tokens
        test_env::set_caller(env.bob);
        env.token.safe_batch_transfer_from(
            env.alice,
            env.bob,
            vec![U256::one(), U256::from(2)],
            vec![50.into(), 100.into()],
            None
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 100.into());
        assert_eq!(env.token.balance_of(env.bob, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.bob, U256::from(2)), 100.into());

        // And the event is emitted
        let contract = env.token;
        assert_events!(
            contract,
            TransferBatch {
                operator: Some(env.bob),
                from: Some(env.alice),
                to: Some(env.bob),
                ids: vec![U256::one(), U256::from(2)],
                values: vec![50.into(), 100.into()]
            }
        );
    }

    #[test]
    fn safe_batch_transfer_errors() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 200.into(), None);
        assert_exception(Error::InsufficientBalance, || {
            // When we transfer more tokens than we have it errors out
            test_env::set_caller(env.alice);
            env.token.safe_batch_transfer_from(
                env.alice,
                env.bob,
                vec![U256::one(), U256::from(2)],
                vec![50.into(), 300.into()],
                None
            );
        });

        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 200.into(), None);
        assert_exception(Error::NotAnOwnerOrApproved, || {
            // When we transfer not our tokens it errors out
            test_env::set_caller(env.bob);
            env.token.safe_batch_transfer_from(
                env.alice,
                env.bob,
                vec![U256::one(), U256::from(2)],
                vec![50.into(), 100.into()],
                None
            );
        });
    }

    #[test]
    fn safe_transfer_to_valid_receiver() {
        // Given a deployed contract
        let mut env = setup();
        // And a valid receiver
        let receiver = Erc1155ReceiverDeployer::default();
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // When we transfer tokens to a valid receiver
        test_env::set_caller(env.alice);
        env.token
            .safe_transfer_from(env.alice, receiver.address(), U256::one(), 100.into(), None);

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 0.into());
        assert_eq!(
            env.token.balance_of(receiver.address(), U256::one()),
            100.into()
        );

        // And receiver contract is aware of received tokens
        assert_events!(
            receiver,
            SingleReceived {
                operator: Some(env.alice),
                from: Some(env.alice),
                token_id: U256::one(),
                amount: 100.into(),
                data: None
            }
        );
    }

    #[test]
    fn safe_transfer_to_valid_receiver_with_data() {
        // Given a deployed contract
        let mut env = setup();
        // And a valid receiver
        let receiver = Erc1155ReceiverDeployer::default();
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // When we transfer tokens to a valid receiver
        test_env::set_caller(env.alice);
        env.token.safe_transfer_from(
            env.alice,
            receiver.address(),
            U256::one(),
            100.into(),
            Some(Bytes::from(b"data".to_vec()))
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 0.into());
        assert_eq!(
            env.token.balance_of(receiver.address(), U256::one()),
            100.into()
        );

        // And receiver contract is aware of received tokens and data
        assert_events!(
            receiver,
            SingleReceived {
                operator: Some(env.alice),
                from: Some(env.alice),
                token_id: U256::one(),
                amount: 100.into(),
                data: Some(Bytes::from(b"data".to_vec()))
            }
        );
    }

    #[test]
    fn safe_transfer_to_invalid_receiver() {
        assert_exception(
            OdraError::VmError(NoSuchMethod("on_erc1155_received".to_string())),
            || {
                // Given a deployed contract
                let mut env = setup();
                // And an invalid receiver
                let receiver = WrappedNativeTokenDeployer::init();
                // And some tokens minted
                env.token.mint(env.alice, U256::one(), 100.into(), None);

                // When we transfer tokens to an invalid receiver
                // Then it errors out
                test_env::set_caller(env.alice);
                env.token.safe_transfer_from(
                    env.alice,
                    receiver.address(),
                    U256::one(),
                    100.into(),
                    None
                );
            }
        );
    }

    #[test]
    #[ignore]
    fn safe_batch_transfer_to_valid_receiver() {
        // Given a deployed contract
        let mut env = setup();
        // And a valid receiver
        let receiver = Erc1155ReceiverDeployer::default();
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 100.into(), None);

        // When we transfer tokens to a valid receiver
        test_env::set_caller(env.alice);
        env.token.safe_batch_transfer_from(
            env.alice,
            receiver.address(),
            vec![U256::one(), U256::from(2)],
            vec![100.into(), 100.into()],
            None
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 0.into());
        assert_eq!(
            env.token.balance_of(receiver.address(), U256::one()),
            100.into()
        );
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 0.into());
        assert_eq!(
            env.token.balance_of(receiver.address(), U256::from(2)),
            100.into()
        );

        // And receiver contract is aware of received tokens
        assert_events!(
            receiver,
            BatchReceived {
                operator: Some(env.alice),
                from: Some(env.alice),
                token_ids: vec![U256::one(), U256::from(2)],
                amounts: vec![100.into(), 100.into()],
                data: None
            }
        );
    }

    #[test]
    #[ignore]
    fn safe_batch_transfer_to_valid_receiver_with_data() {
        // Given a deployed contract
        let mut env = setup();
        // And a valid receiver
        let receiver = Erc1155ReceiverDeployer::default();
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 100.into(), None);

        // When we transfer tokens to a valid receiver
        test_env::set_caller(env.alice);
        env.token.safe_batch_transfer_from(
            env.alice,
            receiver.address(),
            vec![U256::one(), U256::from(2)],
            vec![100.into(), 100.into()],
            Some(Bytes::from(b"data".to_vec()))
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 0.into());
        assert_eq!(
            env.token.balance_of(receiver.address(), U256::one()),
            100.into()
        );
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 0.into());
        assert_eq!(
            env.token.balance_of(receiver.address(), U256::from(2)),
            100.into()
        );

        // And receiver contract is aware of received tokens and data
        assert_events!(
            receiver,
            BatchReceived {
                operator: Some(env.alice),
                from: Some(env.alice),
                token_ids: vec![U256::one(), U256::from(2)],
                amounts: vec![100.into(), 100.into()],
                data: Some(Bytes::from(b"data".to_vec()))
            }
        );
    }

    #[test]
    #[ignore]
    fn safe_batch_transfer_to_invalid_receiver() {
        assert_exception(
            OdraError::VmError(NoSuchMethod("on_erc1155_batch_received".to_string())),
            || {
                // Given a deployed contract
                let mut env = setup();
                // And an invalid receiver
                let receiver = WrappedNativeTokenDeployer::init();
                // And some tokens minted
                env.token.mint(env.alice, U256::one(), 100.into(), None);
                env.token.mint(env.alice, U256::from(2), 100.into(), None);

                // When we transfer tokens to an invalid receiver
                // Then it errors out
                test_env::set_caller(env.alice);
                env.token.safe_batch_transfer_from(
                    env.alice,
                    receiver.address(),
                    vec![U256::one(), U256::from(2)],
                    vec![100.into(), 100.into()],
                    None
                );
            }
        );
    }
}
