//! This examples shows how to handle events in a contract.
use odra::prelude::*;
use Address;

/// Contract that emits an event when initialized.
#[odra::module(events = [PartyStarted, NativePartyStarted], errors = PartyError)]
pub struct PartyContract;

/// Errors of [PartyContract].
#[odra::odra_error]
pub enum PartyError {
    /// The party was cancelled.
    Cancelled = 1
}

/// Event emitted when the contract is initialized.
#[odra::event]
pub struct PartyStarted {
    /// Address of the caller.
    pub caller: Address,
    /// Block time when the contract was initialized.
    pub block_time: u64
}

/// Native version of the above.
#[odra::event]
pub struct NativePartyStarted {
    /// Address of the caller.
    pub caller: Address,
    /// Block time when the contract was initialized.
    pub block_time: u64
}

#[odra::module]
impl PartyContract {
    /// Initializes the contract.
    pub fn init(&self) {
        self.env().emit_event(PartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
        self.env().emit_native_event(NativePartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
    }

    /// Emits the events.
    pub fn emit(&mut self) {
        self.env().emit_event(PartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
        self.env().emit_native_event(NativePartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
    }

    /// Emits the events and reverts, so the events are never emitted.
    pub fn emit_and_revert(&mut self) {
        self.emit();
        self.env().revert(PartyError::Cancelled)
    }
}

/// Contract that emits its own events and then makes [PartyContract] emit.
#[odra::module(events = [PartyStarted])]
pub struct PartyRelay {
    party: External<PartyContractContractRef>
}

#[odra::module]
impl PartyRelay {
    /// Initializes the contract with the address of the party.
    pub fn init(&mut self, party: Address) {
        self.party.set(party);
    }

    /// Emits an event, makes the party emit and, if `cancel` is set, revert.
    pub fn emit(&mut self, cancel: bool) {
        self.env().emit_event(PartyStarted {
            caller: self.env().caller(),
            block_time: self.env().get_block_time()
        });
        if cancel {
            self.party.emit_and_revert();
        } else {
            self.party.emit();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NativePartyStarted, PartyContract, PartyError, PartyRelay, PartyRelayInitArgs, PartyStarted
    };
    use core::time::Duration;
    use odra::host::{Deployer, HostRef, NoArgs};
    use odra::prelude::*;

    #[test]
    fn test_party() {
        let test_env = odra_test::env();
        let mut party_contract = PartyContract::deploy(&test_env, NoArgs);
        assert!(test_env.emitted_event(
            &party_contract,
            PartyStarted {
                caller: test_env.get_account(0),
                block_time: 0
            }
        ));

        assert!(test_env.emitted_native_event(
            &party_contract,
            NativePartyStarted {
                caller: test_env.get_account(0),
                block_time: 0
            }
        ));
        assert!(test_env.emitted(&party_contract, "PartyStarted"));
        assert!(test_env.emitted_native(&party_contract, "NativePartyStarted"));
        assert_eq!(test_env.events_count(&party_contract), 1);
        assert_eq!(test_env.native_events_count(&party_contract), 1);
        test_env.advance_block_time(Duration::from_millis(42));
        test_env.set_caller(test_env.get_account(1));
        party_contract.emit();

        assert!(test_env.emitted_event(
            &party_contract,
            PartyStarted {
                caller: test_env.get_account(1),
                block_time: 42
            }
        ));
        assert!(test_env.emitted_native_event(
            &party_contract,
            NativePartyStarted {
                caller: test_env.get_account(1),
                block_time: 42
            }
        ));
        assert_eq!(test_env.events_count(&party_contract), 2);
        assert_eq!(test_env.native_events_count(&party_contract), 2);
    }

    #[test]
    fn reverted_call_emits_nothing() {
        let test_env = odra_test::env();
        let mut party = PartyContract::deploy(&test_env, NoArgs);
        test_env.advance_block_time(Duration::from_millis(42));
        test_env.set_caller(test_env.get_account(1));
        let caller = test_env.get_account(1);
        let event = || PartyStarted {
            caller,
            block_time: 42
        };
        let native_event = || NativePartyStarted {
            caller,
            block_time: 42
        };

        assert_eq!(
            party.try_emit_and_revert(),
            Err(PartyError::Cancelled.into())
        );
        assert!(party.last_call().events().is_empty());
        assert!(party.last_call().native_events().is_empty());
        assert!(!test_env.emitted_event(&party, event()));
        assert!(!test_env.emitted_native_event(&party, native_event()));
        assert_eq!(test_env.events_count(&party), 1);
        assert_eq!(test_env.native_events_count(&party), 1);

        // The next events come right after the ones of `init`.
        party.emit();
        assert!(party.last_call().emitted_event(event()));
        assert!(party.last_call().emitted_native_event(native_event()));
        assert_eq!(party.last_call().events().len(), 1);
        assert_eq!(test_env.events_count(&party), 2);
        assert_eq!(test_env.native_events_count(&party), 2);
        assert_eq!(test_env.get_event(&party, 1), Ok(event()));
        assert_eq!(test_env.get_native_event(&party, 1), Ok(native_event()));
    }

    #[test]
    fn nested_reverted_call_emits_nothing() {
        let test_env = odra_test::env();
        let mut party = PartyContract::deploy(&test_env, NoArgs);
        let mut relay = PartyRelay::deploy(
            &test_env,
            PartyRelayInitArgs {
                party: party.address()
            }
        );

        // The party reverts, so does the whole call: the relay's event goes too.
        assert_eq!(relay.try_emit(true), Err(PartyError::Cancelled.into()));
        assert!(relay.last_call().events().is_empty());
        assert_eq!(test_env.events_count(&relay), 0);
        assert_eq!(test_env.events_count(&party), 1);
        assert_eq!(test_env.native_events_count(&party), 1);

        relay.emit(false);
        assert_eq!(relay.last_call().events().len(), 1);
        assert_eq!(test_env.events_count(&relay), 1);
        assert_eq!(test_env.events_count(&party), 2);
        assert_eq!(test_env.native_events_count(&party), 2);
        // An event emitted in the call, not the one of `init`.
        party.emit();
        assert_eq!(test_env.events_count(&party), 3);
    }
}

/// An event with a field of an `#[odra::odra_type]` struct cannot be emitted: its CLType is
/// `Any`, which casper-event-standard rejects. Such a contract is test-only here (it is not in
/// `Odra.toml`): it fails when it is deployed, and its schema would fail to generate.
#[cfg(test)]
mod unemittable_events {
    use odra::casper_types::U256;
    use odra::host::{Deployer, NoArgs};
    use odra::prelude::*;

    #[odra::odra_type]
    pub struct Price {
        pub amount: U256,
        pub currency: String
    }

    #[odra::odra_type]
    pub enum Side {
        Buy,
        Sell
    }

    #[odra::event]
    pub struct Traded {
        pub side: Side,
        pub price: Option<Price>
    }

    #[odra::module(events = [Traded])]
    pub struct Exchange;

    #[odra::module]
    impl Exchange {
        pub fn trade(&mut self) {
            self.env().emit_event(Traded {
                side: Side::Buy,
                price: None
            });
        }
    }

    #[test]
    #[should_panic(
        expected = "Contract `Exchange`: event `Traded`: field `price` has no concrete CLType"
    )]
    fn deploying_a_contract_with_an_unemittable_event_panics() {
        Exchange::deploy(&odra_test::env(), NoArgs);
    }

    #[test]
    #[should_panic(
        expected = "Contract `Exchange`: event `Traded`: field `price` has no concrete CLType"
    )]
    fn try_deploy_panics_too() {
        let _ = Exchange::try_deploy(&odra_test::env(), NoArgs);
    }
}
