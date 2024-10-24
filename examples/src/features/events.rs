//! This examples shows how to handle events in a contract.
use odra::prelude::*;
use Address;

/// Contract that emits an event when initialized.
#[odra::module(events = [PartyStarted])]
pub struct PartyContract;

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
}

#[cfg(test)]
mod tests {
    use super::{NativePartyStarted, PartyContract, PartyStarted};
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn test_party() {
        let test_env = odra_test::env();
        let mut party_contract = PartyContract::deploy(&test_env, NoArgs);
        assert!(test_env.emitted_event(
            &party_contract,
            &PartyStarted {
                caller: test_env.get_account(0),
                block_time: 0
            }
        ));
        assert!(test_env.emitted_native_event(
            &party_contract,
            &NativePartyStarted {
                caller: test_env.get_account(0),
                block_time: 0
            }
        ));
        assert_eq!(test_env.events_count(&party_contract), 1);
        assert_eq!(test_env.native_events_count(&party_contract), 1);
        test_env.advance_block_time(42);
        test_env.set_caller(test_env.get_account(1));
        party_contract.emit();

        assert!(test_env.emitted_event(
            &party_contract,
            &PartyStarted {
                caller: test_env.get_account(1),
                block_time: 42
            }
        ));
        assert!(test_env.emitted_native_event(
            &party_contract,
            &NativePartyStarted {
                caller: test_env.get_account(1),
                block_time: 42
            }
        ));
        assert_eq!(test_env.events_count(&party_contract), 2);
        assert_eq!(test_env.native_events_count(&party_contract), 2);
    }
}
