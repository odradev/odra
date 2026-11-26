//! Runs `resources/account_manager.wasm` on the Casper execution engine (the test builder of
//! casper-engine-test-support, with the addressable entity model off, as on mainnet) and checks
//! the account afterwards.

use std::collections::BTreeSet;

use casper_engine_test_support::{
    ExecuteRequestBuilder, LmdbWasmTestBuilder, DEFAULT_ACCOUNT_ADDR, LOCAL_GENESIS_REQUEST
};
use casper_execution_engine::{engine_state, execution::ExecError};
use casper_types::{account::AccountHash, ApiError};
use odra_casper_account_manager::{plan, AccountChange, AccountState, Error, ACCOUNT_MANAGER_WASM};

fn key(n: u8) -> AccountHash {
    AccountHash::new([n; 32])
}

struct Chain {
    builder: LmdbWasmTestBuilder
}

impl Chain {
    fn new() -> Self {
        let mut builder = LmdbWasmTestBuilder::default();
        builder.run_genesis(LOCAL_GENESIS_REQUEST.clone()).commit();
        Chain { builder }
    }

    fn owner() -> AccountHash {
        *DEFAULT_ACCOUNT_ADDR
    }

    /// Sends the change from the default account, signed by `signers`.
    fn apply(
        &mut self,
        change: &AccountChange,
        signers: &[AccountHash]
    ) -> Result<(), engine_state::Error> {
        let request = ExecuteRequestBuilder::module_bytes(
            Self::owner(),
            ACCOUNT_MANAGER_WASM.to_vec(),
            change.to_runtime_args()
        )
        .with_authorization_keys(signers.iter().copied().collect::<BTreeSet<_>>())
        .build();
        self.builder.exec(request).commit();
        match self.builder.get_error() {
            None => Ok(()),
            Some(error) => Err(error)
        }
    }

    fn apply_ok(&mut self, change: &AccountChange, signers: &[AccountHash]) {
        if let Err(error) = self.apply(change, signers) {
            panic!("expected {change:?} to succeed, got {error:?}");
        }
    }

    fn apply_err(&mut self, change: &AccountChange, signers: &[AccountHash], expected: Error) {
        let state = self.state();
        match self.apply(change, signers) {
            Err(engine_state::Error::Exec(ExecError::Revert(ApiError::User(code)))) => {
                assert_eq!(Error::from_code(code), Some(expected), "{change:?}");
            }
            other => panic!("expected {expected:?} for {change:?}, got {other:?}")
        }
        assert_eq!(
            self.state(),
            state,
            "a failed change must leave the account as it was"
        );
    }

    fn state(&self) -> AccountState {
        let account = self
            .builder
            .get_account(Self::owner())
            .expect("the default account exists");
        AccountState {
            associated_keys: account
                .associated_keys()
                .iter()
                .map(|(key, weight)| (*key, weight.value()))
                .collect(),
            deployment: account.action_thresholds().deployment().value(),
            key_management: account.action_thresholds().key_management().value()
        }
    }

    /// Checks the change with `plan` first, applies it and compares the result with the plan.
    fn apply_planned(&mut self, change: &AccountChange, signers: &[AccountHash]) -> AccountState {
        let own: BTreeSet<_> = signers.iter().copied().collect();
        let planned = plan(&self.state(), change, &own).expect("plan accepts the change");
        self.apply_ok(change, signers);
        assert_eq!(self.state(), planned.after);
        planned.after
    }
}

/// The owner with weight 1 and key 2 with weight 1; deployment 1, key_management 2.
fn one_to_send_two_to_manage(chain: &mut Chain) {
    chain.apply_planned(
        &AccountChange::new().set_key(key(2), 1).key_management(2),
        &[Chain::owner()]
    );
}

#[test]
fn a_new_account_has_one_key() {
    let chain = Chain::new();
    assert_eq!(chain.state(), AccountState::new(Chain::owner()));
}

#[test]
fn one_key_sets_up_two_of_two_in_one_transaction() {
    let mut chain = Chain::new();
    let after = chain.apply_planned(
        &AccountChange::new()
            .set_key(key(2), 1)
            .deployment(2)
            .key_management(2),
        &[Chain::owner()]
    );
    assert_eq!(after.associated_keys.len(), 2);
    assert_eq!((after.deployment, after.key_management), (2, 2));

    // The owner alone can no longer send anything from the account.
    let change = AccountChange::new().set_key(key(2), 2);
    assert!(chain.apply(&change, &[Chain::owner()]).is_err());
    // Both keys can.
    chain.apply_planned(&change, &[Chain::owner(), key(2)]);
}

#[test]
fn two_of_two_back_to_a_single_key() {
    let mut chain = Chain::new();
    chain.apply_planned(
        &AccountChange::new()
            .set_key(key(2), 1)
            .deployment(2)
            .key_management(2),
        &[Chain::owner()]
    );
    // Removing key 2 first would break both thresholds, and key_management cannot go below
    // the current deployment: the wasm lowers deployment, then key_management, then removes.
    let after = chain.apply_planned(
        &AccountChange::new()
            .remove_key(key(2))
            .deployment(1)
            .key_management(1),
        &[Chain::owner(), key(2)]
    );
    assert_eq!(after, AccountState::new(Chain::owner()));
}

#[test]
fn a_lowered_weight_waits_for_the_threshold() {
    let mut chain = Chain::new();
    chain.apply_planned(
        &AccountChange::new()
            .set_key(Chain::owner(), 2)
            .set_key(key(2), 1)
            .key_management(3),
        &[Chain::owner()]
    );
    // Lowering the owner to 1 first would take the total (2) below key_management (3).
    let after = chain.apply_planned(
        &AccountChange::new()
            .set_key(Chain::owner(), 1)
            .key_management(2),
        &[Chain::owner(), key(2)]
    );
    assert_eq!(after.associated_keys[&Chain::owner()], 1);
    assert_eq!(after.key_management, 2);
}

#[test]
fn raises_key_management_before_deployment() {
    let mut chain = Chain::new();
    chain.apply_planned(
        &AccountChange::new()
            .set_key(key(2), 1)
            .set_key(key(3), 1)
            .deployment(2)
            .key_management(3),
        &[Chain::owner()]
    );
    let after = chain.state();
    assert_eq!((after.deployment, after.key_management), (2, 3));
}

#[test]
fn argument_errors() {
    let mut chain = Chain::new();
    let owner = [Chain::owner()];
    chain.apply_err(&AccountChange::new(), &owner, Error::EmptyChange);
    chain.apply_err(
        &AccountChange::new().set_key(key(2), 0),
        &owner,
        Error::ZeroWeight
    );
    chain.apply_err(
        &AccountChange::new().deployment(0),
        &owner,
        Error::ZeroThreshold
    );
    chain.apply_err(
        &AccountChange::new().key_management(0),
        &owner,
        Error::ZeroThreshold
    );
    chain.apply_err(
        &AccountChange::new().set_key(key(2), 1).remove_key(key(2)),
        &owner,
        Error::ConflictingKey
    );
}

#[test]
fn threshold_errors() {
    let mut chain = Chain::new();
    let owner = [Chain::owner()];
    chain.apply_err(
        &AccountChange::new().key_management(5),
        &owner,
        Error::ThresholdAboveTotalWeight
    );
    chain.apply_err(
        &AccountChange::new().set_key(key(2), 1).deployment(2),
        &owner,
        Error::DeploymentAboveKeyManagement
    );

    chain.apply_planned(
        &AccountChange::new()
            .set_key(key(2), 1)
            .deployment(2)
            .key_management(2),
        &owner
    );
    let both = [Chain::owner(), key(2)];
    chain.apply_err(
        &AccountChange::new().key_management(1),
        &both,
        Error::KeyManagementBelowDeployment
    );
    chain.apply_err(
        &AccountChange::new().remove_key(key(2)),
        &both,
        Error::RemoveKeyThresholdViolation
    );
}

#[test]
fn a_lowered_weight_that_breaks_a_threshold() {
    let mut chain = Chain::new();
    chain.apply_planned(
        &AccountChange::new().set_key(key(2), 2).key_management(3),
        &[Chain::owner()]
    );
    chain.apply_err(
        &AccountChange::new().set_key(key(2), 1),
        &[Chain::owner(), key(2)],
        Error::UpdateKeyThresholdViolation
    );
}

#[test]
fn key_errors() {
    let mut chain = Chain::new();
    let owner = [Chain::owner()];
    chain.apply_err(
        &AccountChange::new().remove_key(key(9)),
        &owner,
        Error::RemoveMissingKey
    );

    let mut change = AccountChange::new();
    for n in 1..=100u8 {
        change = change.set_key(key(n), 1);
    }
    chain.apply_err(&change, &owner, Error::MaxKeysLimit);
}

#[test]
fn updates_a_key_of_a_full_account() {
    let mut chain = Chain::new();
    let mut change = AccountChange::new();
    for n in 1..=99u8 {
        change = change.set_key(key(n), 1);
    }
    let after = chain.apply_planned(&change, &[Chain::owner()]);
    assert_eq!(after.associated_keys.len(), 100);
    // The engine reports MaxKeysLimit before it looks for the key; the wasm then updates it.
    let after = chain.apply_planned(&AccountChange::new().set_key(key(5), 3), &[Chain::owner()]);
    assert_eq!(after.associated_keys[&key(5)], 3);
}

#[test]
fn signers_below_key_management_are_denied() {
    let mut chain = Chain::new();
    one_to_send_two_to_manage(&mut chain);
    // The owner alone reaches deployment (1) but not key_management (2).
    let owner = [Chain::owner()];
    chain.apply_err(
        &AccountChange::new().set_key(key(3), 1),
        &owner,
        Error::AddKeyPermissionDenied
    );
    // The wasm tries to add an existing key first, and the engine checks the permission before
    // it looks for the key.
    chain.apply_err(
        &AccountChange::new().set_key(key(2), 3),
        &owner,
        Error::AddKeyPermissionDenied
    );
    chain.apply_err(
        &AccountChange::new().remove_key(key(2)),
        &owner,
        Error::RemoveKeyPermissionDenied
    );
    chain.apply_err(
        &AccountChange::new().deployment(1),
        &owner,
        Error::SetThresholdPermissionDenied
    );
}
