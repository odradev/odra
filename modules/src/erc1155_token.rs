//! A pluggable Odra module implementing Erc1155 token with ownership.
use crate::access::Ownable;
use crate::erc1155::erc1155_base::Erc1155Base;
use crate::erc1155::errors::Error;
use crate::erc1155::events::{TransferBatch, TransferSingle};
use crate::erc1155::owned_erc1155::OwnedErc1155;
use crate::erc1155::Erc1155;
use odra::prelude::*;
use odra::{
    casper_types::{bytesrepr::Bytes, U256},
    module::{Module, ModuleWrapper},
    Address
};

/// The ERC1155 token implementation.
/// It uses the [ERC1155](Erc1155Base) base implementation and the [Ownable] module.
#[odra::module(events = [TransferBatch, TransferSingle])]
pub struct Erc1155Token {
    core: ModuleWrapper<Erc1155Base>,
    ownable: ModuleWrapper<Ownable>
}

#[odra::module]
impl OwnedErc1155 for Erc1155Token {
    fn init(&mut self) {
        self.ownable.init();
    }

    fn balance_of(&self, owner: &Address, id: &U256) -> U256 {
        self.core.balance_of(owner, id)
    }
    fn balance_of_batch(&self, owners: &[Address], ids: &[U256]) -> Vec<U256> {
        self.core.balance_of_batch(owners, ids)
    }
    fn set_approval_for_all(&mut self, operator: &Address, approved: bool) {
        self.core.set_approval_for_all(operator, approved);
    }
    fn is_approved_for_all(&self, owner: &Address, operator: &Address) -> bool {
        self.core.is_approved_for_all(owner, operator)
    }
    fn safe_transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        id: &U256,
        amount: &U256,
        data: &Option<Bytes>
    ) {
        self.core.safe_transfer_from(from, to, id, amount, data);
    }

    fn safe_batch_transfer_from(
        &mut self,
        from: &Address,
        to: &Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
        data: &Option<Bytes>
    ) {
        self.core
            .safe_batch_transfer_from(from, to, ids, amounts, data);
    }

    // Ownable
    fn renounce_ownership(&mut self) {
        self.ownable.renounce_ownership();
    }

    fn transfer_ownership(&mut self, new_owner: &Address) {
        self.ownable.transfer_ownership(new_owner);
    }

    fn owner(&self) -> Address {
        self.ownable.get_owner()
    }

    fn mint(&mut self, to: &Address, id: &U256, amount: &U256, data: &Option<Bytes>) {
        let caller = self.env().caller();
        self.ownable.assert_owner(&caller);

        let current_balance = self.core.balances.get_or_default(&(*to, *id));
        self.core
            .balances
            .set(&(*to, *id), *amount + current_balance);

        self.env().emit_event(TransferSingle {
            operator: Some(caller),
            from: None,
            to: Some(*to),
            id: *id,
            value: *amount
        });

        self.core
            .safe_transfer_acceptance_check(&caller, &caller, to, id, amount, data);
    }

    fn mint_batch(
        &mut self,
        to: &Address,
        ids: Vec<U256>,
        amounts: Vec<U256>,
        data: &Option<Bytes>
    ) {
        if ids.len() != amounts.len() {
            self.env().revert(Error::IdsAndAmountsLengthMismatch)
        }

        let caller = self.env().caller();
        self.ownable.assert_owner(&caller);

        for (id, amount) in ids.iter().zip(amounts.iter()) {
            let current_balance = self.core.balances.get_or_default(&(*to, *id));
            self.core
                .balances
                .set(&(*to, *id), *amount + current_balance);
        }

        self.env().emit_event(TransferBatch {
            operator: Some(caller),
            from: None,
            to: Some(*to),
            ids: ids.to_vec(),
            values: amounts.to_vec()
        });

        self.core
            .safe_batch_transfer_acceptance_check(&caller, &caller, to, ids, amounts, data);
    }

    fn burn(&mut self, from: &Address, id: &U256, amount: &U256) {
        let caller = self.env().caller();
        self.ownable.assert_owner(&caller);

        let current_balance = self.core.balances.get_or_default(&(*from, *id));

        if current_balance < *amount {
            self.env().revert(Error::InsufficientBalance)
        }

        self.core
            .balances
            .set(&(*from, *id), current_balance - *amount);

        self.env().emit_event(TransferSingle {
            operator: Some(caller),
            from: Some(*from),
            to: None,
            id: *id,
            value: *amount
        });
    }

    fn burn_batch(&mut self, from: &Address, ids: Vec<U256>, amounts: Vec<U256>) {
        if ids.len() != amounts.len() {
            self.env().revert(Error::IdsAndAmountsLengthMismatch)
        }

        let caller = self.env().caller();
        self.ownable.assert_owner(&caller);

        for (id, amount) in ids.iter().zip(amounts.iter()) {
            let current_balance = self.core.balances.get_or_default(&(*from, *id));

            if current_balance < *amount {
                self.env().revert(Error::InsufficientBalance)
            }
            self.core
                .balances
                .set(&(*from, *id), current_balance - *amount);
        }

        self.env().emit_event(TransferBatch {
            operator: Some(caller),
            from: Some(*from),
            to: None,
            ids: ids.to_vec(),
            values: amounts.to_vec()
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::erc1155::errors::Error;
    use crate::erc1155::errors::Error::InsufficientBalance;
    use crate::erc1155::events::{ApprovalForAll, TransferBatch, TransferSingle};
    use crate::erc1155_receiver::events::{BatchReceived, SingleReceived};
    use crate::erc1155_receiver::{Erc1155ReceiverHostRef, Erc1155ReceiverInitArgs};
    use crate::erc1155_token::Erc1155TokenHostRef;
    use crate::wrapped_native::{WrappedNativeTokenHostRef, WrappedNativeTokenInitArgs};
    use odra::host::{Deployer, HostEnv, HostRef};
    use odra::prelude::*;
    use odra::{
        casper_types::{bytesrepr::Bytes, U256},
        Address, OdraError, VmError
    };

    use super::Erc1155TokenInitArgs;

    struct TokenEnv {
        env: HostEnv,
        token: Erc1155TokenHostRef,
        alice: Address,
        bob: Address,
        owner: Address
    }

    fn setup() -> TokenEnv {
        let env = odra_test::env();

        TokenEnv {
            env: env.clone(),
            token: Erc1155TokenHostRef::deploy(&env, Erc1155TokenInitArgs),
            alice: env.get_account(1),
            bob: env.get_account(2),
            owner: env.get_account(0)
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
        env.env.emitted_event(
            contract.address(),
            &TransferSingle {
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
            [U256::one(), U256::from(2)].to_vec(),
            [100.into(), 200.into()].to_vec(),
            None
        );

        // Then it emits the event
        env.env.emitted_event(
            env.token.address(),
            &TransferBatch {
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
        // Given a deployed contract
        let mut env = setup();
        // When we mint some tokens in batch with mismatching ids and amounts it errors out
        let err = env
            .token
            .try_mint_batch(
                env.alice,
                [U256::one(), U256::from(2)].to_vec(),
                [100.into()].to_vec(),
                None
            )
            .unwrap_err();
        assert_eq!(err, Error::IdsAndAmountsLengthMismatch.into());
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
        env.env.emitted_event(
            contract.address(),
            &TransferSingle {
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
        // When we burn more tokens than we have it errors out
        let err = env
            .token
            .try_burn(env.alice, U256::one(), 150.into())
            .unwrap_err();
        assert_eq!(err, InsufficientBalance.into());

        // Given a deployed contract
        let mut env = setup();
        // When we burn non-existing tokens it errors out
        let err = env
            .token
            .try_burn(env.alice, U256::one(), 150.into())
            .unwrap_err();
        assert_eq!(err, InsufficientBalance.into());
    }

    #[test]
    fn burn_batch() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint_batch(
            env.alice,
            [U256::one(), U256::from(2)].to_vec(),
            [100.into(), 200.into()].to_vec(),
            None
        );

        // When we burn some tokens in batch
        env.token.burn_batch(
            env.alice,
            [U256::one(), U256::from(2)].to_vec(),
            [50.into(), 100.into()].to_vec()
        );

        // Then the balances are updated
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 100.into());

        // And the event is emitted
        let contract = env.token;
        env.env.emitted_event(
            contract.address(),
            &TransferBatch {
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
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint_batch(
            env.alice,
            [U256::one(), U256::from(2)].to_vec(),
            [100.into(), 200.into()].to_vec(),
            None
        );

        // When we burn some tokens in batch with mismatching ids and amounts it errors out
        let err = env
            .token
            .try_burn_batch(
                env.alice,
                [U256::one(), U256::from(2)].to_vec(),
                [50.into()].to_vec()
            )
            .unwrap_err();
        assert_eq!(err, Error::IdsAndAmountsLengthMismatch.into());

        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint_batch(
            env.alice,
            [U256::one(), U256::from(2)].to_vec(),
            [100.into(), 200.into()].to_vec(),
            None
        );

        // When we burn more tokens than we have it errors out
        let err = env
            .token
            .try_burn_batch(
                env.alice,
                [U256::one(), U256::from(2)].to_vec(),
                [150.into(), 300.into()].to_vec()
            )
            .unwrap_err();
        assert_eq!(err, InsufficientBalance.into());

        // Given a deployed contract
        let mut env = setup();

        // When we burn non-existing tokens it errors out
        let err = env
            .token
            .try_burn_batch(
                env.alice,
                [U256::one(), U256::from(2)].to_vec(),
                [150.into(), 300.into()].to_vec()
            )
            .unwrap_err();
        assert_eq!(err, InsufficientBalance.into());
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
            [U256::one(), U256::from(2)].to_vec(),
            [100.into(), 200.into()].to_vec(),
            None
        );
        env.token.mint_batch(
            env.bob,
            [U256::one(), U256::from(2)].to_vec(),
            [300.into(), 400.into()].to_vec(),
            None
        );

        // Then it returns the correct balances
        assert_eq!(
            env.token.balance_of_batch(
                [env.alice, env.alice, env.alice, env.bob, env.bob, env.bob].to_vec(),
                [
                    U256::one(),
                    U256::from(2),
                    U256::from(3),
                    U256::one(),
                    U256::from(2),
                    U256::from(3)
                ]
                .to_vec()
            ),
            // TODO: Why it gives deserialization error when mismatched?
            [
                U256::from(100),
                U256::from(200),
                U256::zero(),
                U256::from(300),
                U256::from(400),
                U256::zero()
            ]
            .to_vec()
        );
    }

    #[test]
    fn balance_of_batch_errors() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint_batch(
            env.alice,
            [U256::one(), U256::from(2)].to_vec(),
            [100.into(), 200.into()].to_vec(),
            None
        );

        // When we query balances with mismatching ids and addresses it errors out
        let err = env
            .token
            .try_balance_of_batch(
                [env.alice, env.alice, env.alice].to_vec(),
                [U256::one(), U256::from(2)].to_vec()
            )
            .unwrap_err();
        assert_eq!(err, Error::AccountsAndIdsLengthMismatch.into());
    }

    #[test]
    fn set_approval_for_all() {
        // Given a deployed contract
        let mut env = setup();

        // When we set approval for all
        env.env.set_caller(env.alice);
        env.token.set_approval_for_all(env.bob, true);

        // Then the approval is set
        assert!(env.token.is_approved_for_all(env.alice, env.bob));

        // And the event is emitted
        env.env.emitted_event(
            env.token.address(),
            &ApprovalForAll {
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
        env.env.set_caller(env.alice);
        env.token.set_approval_for_all(env.bob, true);

        // When we unset approval for all
        env.env.set_caller(env.alice);
        env.token.set_approval_for_all(env.bob, false);

        // Then the approval is unset
        assert!(!env.token.is_approved_for_all(env.alice, env.bob));

        // And the event is emitted
        let contract = env.token;
        env.env.emitted_event(
            contract.address(),
            &ApprovalForAll {
                owner: env.alice,
                operator: env.bob,
                approved: false
            }
        );
    }

    #[test]
    fn set_approval_to_self() {
        // Given a deployed contract
        let mut env = setup();

        // Then approving for self throws an error
        env.env.set_caller(env.alice);
        let err = env
            .token
            .try_set_approval_for_all(env.alice, true)
            .unwrap_err();
        assert_eq!(err, Error::ApprovalForSelf.into());
    }

    #[test]
    fn safe_transfer_from() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // When we transfer tokens
        env.env.set_caller(env.alice);
        env.token
            .safe_transfer_from(env.alice, env.bob, U256::one(), 50.into(), None);

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.bob, U256::one()), 50.into());

        // And the event is emitted
        let contract = env.token;
        env.env.emitted_event(
            contract.address(),
            &TransferSingle {
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
        env.env.set_caller(env.alice);
        env.token.set_approval_for_all(env.bob, true);

        // When we transfer tokens
        env.env.set_caller(env.bob);
        env.token
            .safe_transfer_from(env.alice, env.bob, U256::one(), 50.into(), None);

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.bob, U256::one()), 50.into());

        // And the event is emitted
        let contract = env.token;
        env.env.emitted_event(
            contract.address(),
            &TransferSingle {
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
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // When we transfer more tokens than we have it errors out
        env.env.set_caller(env.alice);
        let err = env
            .token
            .try_safe_transfer_from(env.alice, env.bob, U256::one(), 200.into(), None)
            .unwrap_err();
        assert_eq!(err, Error::InsufficientBalance.into());

        // Given a deployed contract
        // env.env.set_caller(test_env::get_account(0));
        let mut env = setup();
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // When we transfer not our tokens it errors out
        env.env.set_caller(env.bob);
        let err = env
            .token
            .try_safe_transfer_from(env.alice, env.bob, U256::one(), 100.into(), None)
            .unwrap_err();
        assert_eq!(err, Error::NotAnOwnerOrApproved.into());
    }

    #[test]
    fn safe_batch_transfer_from() {
        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 200.into(), None);

        // When we transfer tokens
        env.env.set_caller(env.alice);
        env.token.safe_batch_transfer_from(
            env.alice,
            env.bob,
            [U256::one(), U256::from(2)].to_vec(),
            [50.into(), 100.into()].to_vec(),
            None
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 100.into());
        assert_eq!(env.token.balance_of(env.bob, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.bob, U256::from(2)), 100.into());

        // And the event is emitted
        let contract = env.token;
        env.env.emitted_event(
            contract.address(),
            &TransferBatch {
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
        env.env.set_caller(env.alice);
        env.token.set_approval_for_all(env.bob, true);

        // When we transfer tokens
        env.env.set_caller(env.bob);
        env.token.safe_batch_transfer_from(
            env.alice,
            env.bob,
            [U256::one(), U256::from(2)].to_vec(),
            [50.into(), 100.into()].to_vec(),
            None
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 100.into());
        assert_eq!(env.token.balance_of(env.bob, U256::one()), 50.into());
        assert_eq!(env.token.balance_of(env.bob, U256::from(2)), 100.into());

        // And the event is emitted
        let contract = env.token;
        env.env.emitted_event(
            contract.address(),
            &TransferBatch {
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
        // When we transfer more tokens than we have it errors out
        env.env.set_caller(env.alice);
        let err = env
            .token
            .try_safe_batch_transfer_from(
                env.alice,
                env.bob,
                [U256::one(), U256::from(2)].to_vec(),
                [50.into(), 300.into()].to_vec(),
                None
            )
            .unwrap_err();
        assert_eq!(err, Error::InsufficientBalance.into());

        // Given a deployed contract
        let mut env = setup();

        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 200.into(), None);
        // When we transfer not our tokens it errors out
        env.env.set_caller(env.bob);
        let err = env
            .token
            .try_safe_batch_transfer_from(
                env.alice,
                env.bob,
                [U256::one(), U256::from(2)].to_vec(),
                [50.into(), 100.into()].to_vec(),
                None
            )
            .unwrap_err();
        assert_eq!(err, Error::NotAnOwnerOrApproved.into());
    }

    #[test]
    fn safe_transfer_to_valid_receiver() {
        // Given a deployed contract
        let mut env = setup();
        // And a valid receiver
        
        let receiver = Erc1155ReceiverHostRef::deploy(&env.env, Erc1155TokenInitArgs);
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // When we transfer tokens to a valid receiver
        env.env.set_caller(env.alice);
        env.token.safe_transfer_from(
            env.alice,
            *receiver.address(),
            U256::one(),
            100.into(),
            None
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 0.into());
        assert_eq!(
            env.token.balance_of(*receiver.address(), U256::one()),
            100.into()
        );

        // And receiver contract is aware of received tokens
        env.env.emitted_event(
            receiver.address(),
            &SingleReceived {
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
        let receiver = Erc1155ReceiverHostRef::deploy(&env.env, Erc1155TokenInitArgs);
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // When we transfer tokens to a valid receiver
        env.env.set_caller(env.alice);
        env.token.safe_transfer_from(
            env.alice,
            *receiver.address(),
            U256::one(),
            100.into(),
            Some(Bytes::from(b"data".to_vec()))
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 0.into());
        assert_eq!(
            env.token.balance_of(*receiver.address(), U256::one()),
            100.into()
        );

        // And receiver contract is aware of received tokens and data
        env.env.emitted_event(
            receiver.address(),
            &SingleReceived {
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
        // Given a deployed contract
        let mut env = setup();
        // And an invalid receiver
        let receiver = WrappedNativeTokenHostRef::deploy(&env.env, WrappedNativeTokenInitArgs);
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);

        // When we transfer tokens to an invalid receiver
        // Then it errors out
        env.env.set_caller(env.alice);
        let err = env.token.try_safe_transfer_from(
            env.alice,
            *receiver.address(),
            U256::one(),
            100.into(),
            None
        );
        assert_eq!(
            err,
            Err(OdraError::VmError(VmError::NoSuchMethod(
                "on_erc1155_received".to_string()
            )))
        );
    }

    #[test]
    fn safe_batch_transfer_to_valid_receiver() {
        // Given a deployed contract
        let mut env = setup();
        // And a valid receiver
        let receiver = Erc1155ReceiverHostRef::deploy(&env.env, Erc1155ReceiverInitArgs);
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 100.into(), None);

        // When we transfer tokens to a valid receiver
        env.env.set_caller(env.alice);
        env.token.safe_batch_transfer_from(
            env.alice,
            *receiver.address(),
            [U256::one(), U256::from(2)].to_vec(),
            [100.into(), 100.into()].to_vec(),
            None
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 0.into());
        assert_eq!(
            env.token.balance_of(*receiver.address(), U256::one()),
            100.into()
        );
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 0.into());
        assert_eq!(
            env.token.balance_of(*receiver.address(), U256::from(2)),
            100.into()
        );

        // And receiver contract is aware of received tokens
        env.env.emitted_event(
            receiver.address(),
            &BatchReceived {
                operator: Some(env.alice),
                from: Some(env.alice),
                token_ids: [U256::one(), U256::from(2)].to_vec(),
                amounts: [100.into(), 100.into()].to_vec(),
                data: None
            }
        );
    }

    #[test]
    fn safe_batch_transfer_to_valid_receiver_with_data() {
        // Given a deployed contract
        let mut env = setup();
        // And a valid receiver
        let receiver = Erc1155ReceiverHostRef::deploy(&env.env, Erc1155ReceiverInitArgs);
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 100.into(), None);

        // When we transfer tokens to a valid receiver
        env.env.set_caller(env.alice);
        env.token.safe_batch_transfer_from(
            env.alice,
            *receiver.address(),
            [U256::one(), U256::from(2)].to_vec(),
            [100.into(), 100.into()].to_vec(),
            Some(Bytes::from(b"data".to_vec()))
        );

        // Then the tokens are transferred
        assert_eq!(env.token.balance_of(env.alice, U256::one()), 0.into());
        assert_eq!(
            env.token.balance_of(*receiver.address(), U256::one()),
            100.into()
        );
        assert_eq!(env.token.balance_of(env.alice, U256::from(2)), 0.into());
        assert_eq!(
            env.token.balance_of(*receiver.address(), U256::from(2)),
            100.into()
        );

        // And receiver contract is aware of received tokens and data
        env.env.emitted_event(
            receiver.address(),
            &BatchReceived {
                operator: Some(env.alice),
                from: Some(env.alice),
                token_ids: [U256::one(), U256::from(2)].to_vec(),
                amounts: [100.into(), 100.into()].to_vec(),
                data: Some(Bytes::from(b"data".to_vec()))
            }
        );
    }

    #[test]
    fn safe_batch_transfer_to_invalid_receiver() {
        // Given a deployed contract
        let mut env = setup();
        // And an invalid receiver
        let receiver = WrappedNativeTokenHostRef::deploy(&env.env, WrappedNativeTokenInitArgs);
        // And some tokens minted
        env.token.mint(env.alice, U256::one(), 100.into(), None);
        env.token.mint(env.alice, U256::from(2), 100.into(), None);

        // When we transfer tokens to an invalid receiver
        // Then it errors out
        env.env.set_caller(env.alice);
        let err = env
            .token
            .try_safe_batch_transfer_from(
                env.alice,
                *receiver.address(),
                [U256::one(), U256::from(2)].to_vec(),
                [100.into(), 100.into()].to_vec(),
                None
            )
            .unwrap_err();
        assert_eq!(
            err,
            OdraError::VmError(VmError::NoSuchMethod(
                "on_erc1155_batch_received".to_string()
            ))
        );
    }
}
