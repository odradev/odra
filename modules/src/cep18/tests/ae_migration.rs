//! Scenarios verifying that CEP-18 state written **before** the
//! `enable_addressable_entity` switch keeps working **after** it.
//!
//! Each test builds pre-switch state, then calls
//! [`odra::host::HostEnv::enable_addressable_entity`]. The switch reopens the
//! chain state with an entity-enabled configuration and performs a protocol
//! upgrade, exactly like a real network migration; contracts and accounts are
//! migrated lazily on first use afterwards. When the switch is unavailable
//! (OdraVM backend, or a CasperVm already running in addressable-entity mode)
//! the test returns early - run the casper backend with
//! `ODRA_CASPER_LEGACY_GENESIS=1` to execute these scenarios.
#[cfg(test)]
mod ae_migration_tests {
    use odra::casper_types::U256;
    use odra::host::{Deployer, HostRef, NoArgs};
    use odra::prelude::Addressable;

    use crate::cep18::cep18_client_contract::Cep18ClientContract;
    use crate::cep18::events::Transfer;
    use crate::cep18_token::tests::{
        invert_address, setup, ALLOWANCE_AMOUNT_1, TOKEN_TOTAL_SUPPLY, TRANSFER_AMOUNT_1
    };

    /// Balances and allowances keyed by account addresses pre-switch must
    /// resolve post-switch, and accounts must keep operating on them.
    #[test]
    fn account_balances_and_allowances_survive_ae_switch() {
        let mut cep18_token = setup();
        let env = cep18_token.env().clone();
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let bob = env.get_account(2);

        // Pre-switch state: a balance and an allowance keyed by account addresses.
        cep18_token.transfer(&alice, &TRANSFER_AMOUNT_1.into());
        cep18_token.approve(&alice, &ALLOWANCE_AMOUNT_1.into());

        if !env.enable_addressable_entity() {
            return;
        }

        // Pre-switch dictionary entries must resolve after the switch.
        assert_eq!(cep18_token.balance_of(&alice), TRANSFER_AMOUNT_1.into());
        assert_eq!(
            cep18_token.balance_of(&owner),
            (TOKEN_TOTAL_SUPPLY - TRANSFER_AMOUNT_1).into()
        );
        assert_eq!(
            cep18_token.allowance(&owner, &alice),
            ALLOWANCE_AMOUNT_1.into()
        );
        // Account/Contract variants must still not collide on raw bytes.
        assert_eq!(cep18_token.balance_of(&invert_address(alice)), 0.into());

        // A fresh transfer post-switch must debit the pre-switch balance.
        cep18_token.transfer(&bob, &TRANSFER_AMOUNT_1.into());
        assert_eq!(cep18_token.balance_of(&bob), TRANSFER_AMOUNT_1.into());
        assert_eq!(
            cep18_token.balance_of(&owner),
            (TOKEN_TOTAL_SUPPLY - 2 * TRANSFER_AMOUNT_1).into()
        );

        // Spending a pre-switch allowance post-switch must work and must key
        // the caller (alice) to the same account address as before the switch.
        env.set_caller(alice);
        cep18_token.transfer_from(&owner, &bob, &ALLOWANCE_AMOUNT_1.into());
        assert_eq!(cep18_token.allowance(&owner, &alice), 0.into());
        assert_eq!(
            cep18_token.balance_of(&bob),
            (TRANSFER_AMOUNT_1 + ALLOWANCE_AMOUNT_1).into()
        );
    }

    /// A contract holding tokens and calling the token pre-switch must map to
    /// the same package-based address post-switch: pre-switch balances keyed by
    /// the caller-derived contract address must be found and updated by
    /// post-switch contract-initiated calls (legacy `CallerInfo` kind 4 vs
    /// addressable-entity kind 3 must normalize identically).
    #[test]
    fn contract_caller_address_survives_ae_switch() {
        let mut cep18_token = setup();
        let env = cep18_token.env().clone();
        let owner = env.get_account(0);
        let alice = env.get_account(1);
        let client_contract = Cep18ClientContract::deploy(&env, NoArgs);

        // Pre-switch: the client contract holds tokens...
        cep18_token.transfer(&client_contract.address(), &(2 * TRANSFER_AMOUNT_1).into());
        // ...spends some of them itself (its address is derived from the call
        // stack inside the token contract)...
        client_contract.transfer_as_stored_contract(
            cep18_token.address(),
            alice,
            TRANSFER_AMOUNT_1.into()
        );
        assert_eq!(
            cep18_token.balance_of(&client_contract.address()),
            TRANSFER_AMOUNT_1.into()
        );
        // ...and receives an allowance from the owner.
        cep18_token.approve(&client_contract.address(), &ALLOWANCE_AMOUNT_1.into());

        if !env.enable_addressable_entity() {
            return;
        }

        // Pre-switch state keyed by the contract's address must resolve.
        assert_eq!(
            cep18_token.balance_of(&client_contract.address()),
            TRANSFER_AMOUNT_1.into()
        );
        assert_eq!(
            cep18_token.allowance(&owner, &client_contract.address()),
            ALLOWANCE_AMOUNT_1.into()
        );
        assert_eq!(
            cep18_token.balance_of(&invert_address(client_contract.address())),
            0.into()
        );

        // Contract-to-contract queries must read pre-switch entries.
        assert_eq!(
            client_contract.check_balance_of(cep18_token.address(), client_contract.address()),
            TRANSFER_AMOUNT_1.into()
        );

        // Post-switch, the contract spending its own tokens must debit its
        // pre-switch balance - the caller must normalize to the same address.
        client_contract.transfer_as_stored_contract(
            cep18_token.address(),
            alice,
            TRANSFER_AMOUNT_1.into()
        );
        assert_eq!(cep18_token.balance_of(&client_contract.address()), 0.into());
        assert_eq!(
            cep18_token.balance_of(&alice),
            (2 * TRANSFER_AMOUNT_1).into()
        );

        // Spending the pre-switch allowance as a contract must work as well.
        client_contract.transfer_from_as_stored_contract(
            cep18_token.address(),
            owner,
            alice,
            ALLOWANCE_AMOUNT_1.into()
        );
        assert_eq!(
            cep18_token.allowance(&owner, &client_contract.address()),
            0.into()
        );
        assert_eq!(
            cep18_token.balance_of(&alice),
            U256::from(2 * TRANSFER_AMOUNT_1) + U256::from(ALLOWANCE_AMOUNT_1)
        );
    }

    /// Events emitted pre-switch must stay readable post-switch, and the
    /// lazily-migrated contract must keep emitting readable events.
    #[test]
    fn events_survive_ae_switch() {
        let mut cep18_token = setup();
        let env = cep18_token.env().clone();
        let owner = env.get_account(0);
        let alice = env.get_account(1);

        cep18_token.transfer(&alice, &TRANSFER_AMOUNT_1.into());
        let pre_switch_events_count = env.events_count(&cep18_token);

        if !env.enable_addressable_entity() {
            return;
        }

        // Pre-switch events are still readable.
        assert_eq!(env.events_count(&cep18_token), pre_switch_events_count);
        assert!(env.emitted_event(
            &cep18_token,
            Transfer {
                sender: owner,
                recipient: alice,
                amount: TRANSFER_AMOUNT_1.into()
            }
        ));

        // Post-switch emissions on the migrated contract work and are readable.
        cep18_token.transfer(&alice, &1.into());
        assert_eq!(env.events_count(&cep18_token), pre_switch_events_count + 1);
        assert!(env.emitted_event(
            &cep18_token,
            Transfer {
                sender: owner,
                recipient: alice,
                amount: 1.into()
            }
        ));
    }
}
