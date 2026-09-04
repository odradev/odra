//! This example shows how to configure a contract's event mode.
//!
//! `self.env().emit_event(..)` is the single API for emitting events; which mechanism(s) it
//! uses is a property of the contract, configured once via `#[odra::module(event_mode = ..)]`
//! on the module's `impl` block, not chosen at each call site. There are three modes:
//! - unset (the default): events go out only through CES (`casper-event-standard`).
//! - `event_mode = native`: events go out only through the Casper native mechanism.
//! - `event_mode = both`: every event is emitted through both mechanisms, as one event, not
//!   two independently constructed ones.
use odra::prelude::*;
use Address;

/// Event emitted when the party starts, however the contract is configured to emit it.
#[odra::event]
pub struct PartyStarted {
    /// Address of the caller.
    pub caller: Address,
    /// Block time when the contract was initialized.
    pub block_time: u64
}

/// A contract left at the default event mode: `emit_event` publishes CES events only.
#[odra::module(events = [PartyStarted])]
pub struct CesPartyContract;

#[odra::module]
impl CesPartyContract {
    /// Initializes the contract, emitting a `PartyStarted` CES event.
    pub fn init(&self) {
        self.env().emit_event(PartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
    }

    /// Emits another `PartyStarted` CES event.
    pub fn emit(&mut self) {
        self.env().emit_event(PartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
    }
}

/// A contract configured for `event_mode = native`: `emit_event` publishes native events only.
#[odra::module(events = [PartyStarted], event_mode = native)]
pub struct NativePartyContract;

#[odra::module(event_mode = native)]
impl NativePartyContract {
    /// Initializes the contract, emitting a `PartyStarted` native event.
    pub fn init(&self) {
        self.env().emit_event(PartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
    }

    /// Emits another `PartyStarted` native event.
    pub fn emit(&mut self) {
        self.env().emit_event(PartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
    }
}

/// A contract configured for `event_mode = both`: `emit_event` publishes the same event
/// through both mechanisms.
#[odra::module(events = [PartyStarted], event_mode = both)]
pub struct BothPartyContract;

#[odra::module(event_mode = both)]
impl BothPartyContract {
    /// Initializes the contract, emitting a `PartyStarted` event via both mechanisms.
    pub fn init(&self) {
        self.env().emit_event(PartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
    }

    /// Emits another `PartyStarted` event via both mechanisms.
    pub fn emit(&mut self) {
        self.env().emit_event(PartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{BothPartyContract, CesPartyContract, NativePartyContract, PartyStarted};
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn default_mode_emits_ces_only() {
        let test_env = odra_test::env();
        let mut contract = CesPartyContract::deploy(&test_env, NoArgs);

        assert!(test_env.emitted_event(
            &contract,
            PartyStarted {
                caller: test_env.get_account(0),
                block_time: 0
            }
        ));
        assert_eq!(test_env.events_count(&contract), 1);
        // No mechanism other than the configured one was used.
        assert_eq!(test_env.native_events_count(&contract), 0);

        test_env.advance_block_time(42);
        test_env.set_caller(test_env.get_account(1));
        contract.emit();

        assert!(test_env.emitted_event(
            &contract,
            PartyStarted {
                caller: test_env.get_account(1),
                block_time: 42
            }
        ));
        assert_eq!(test_env.events_count(&contract), 2);
        assert_eq!(test_env.native_events_count(&contract), 0);
    }

    #[test]
    fn native_mode_emits_native_only() {
        let test_env = odra_test::env();
        let mut contract = NativePartyContract::deploy(&test_env, NoArgs);

        assert!(test_env.emitted_native_event(
            &contract,
            PartyStarted {
                caller: test_env.get_account(0),
                block_time: 0
            }
        ));
        assert_eq!(test_env.native_events_count(&contract), 1);
        // CES was not used.
        assert_eq!(test_env.events_count(&contract), 0);

        test_env.advance_block_time(42);
        test_env.set_caller(test_env.get_account(1));
        contract.emit();

        assert!(test_env.emitted_native_event(
            &contract,
            PartyStarted {
                caller: test_env.get_account(1),
                block_time: 42
            }
        ));
        assert_eq!(test_env.native_events_count(&contract), 2);
        assert_eq!(test_env.events_count(&contract), 0);
    }

    #[test]
    fn both_mode_emits_the_same_event_twice_once_per_mechanism() {
        let test_env = odra_test::env();
        let contract = BothPartyContract::deploy(&test_env, NoArgs);

        let caller = test_env.get_account(0);
        assert!(test_env.emitted_event(
            &contract,
            PartyStarted {
                caller,
                block_time: 0
            }
        ));
        assert!(test_env.emitted_native_event(
            &contract,
            PartyStarted {
                caller,
                block_time: 0
            }
        ));
        assert_eq!(test_env.events_count(&contract), 1);
        assert_eq!(test_env.native_events_count(&contract), 1);
    }
}
