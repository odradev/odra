//! The events of the contracts a [`HostEnv`] knows: reading them from the backend, and the
//! per-contract counters that attribute the events of each call to its `last_call()`.

use crate::address::Addressable;
use crate::host::deployed_contracts::DeployedContract;
use crate::host::HostEnv;
use crate::{prelude::*, utils, EventError};
use casper_event_standard::EventInstance;
use casper_types::bytesrepr::{Bytes, FromBytes, ToBytes};

impl HostEnv {
    /// Sets the `captures_events` flag, which determines whether events should be captured.
    pub fn set_captures_events(&self, captures: bool) {
        *self.captures_events.borrow_mut() = captures;
        if captures {
            // Initialize events for all deployed contracts if capturing is enabled
            let contract_addresses: Vec<Address> =
                self.deployed_contracts.borrow().keys().copied().collect();

            for contract_address in contract_addresses {
                self.init_events(&contract_address);
            }
        }
    }

    /// Retrieves an event with the specified index from the specified contract.
    ///
    /// # Returns
    ///
    /// Returns the event as an instance of the specified type, or an error if the event
    /// couldn't be retrieved or parsed.
    pub fn get_event<T: FromBytes + EventInstance, R: Addressable>(
        &self,
        addr: &R,
        index: i32
    ) -> Result<T, EventError> {
        let contract_address = addr.address();
        let backend = self.backend.as_ref();
        let events_count = self.events_count(&contract_address);
        let event_absolute_position = crate::utils::event_absolute_position(events_count, index)
            .ok_or(EventError::IndexOutOfBounds)?;

        let bytes = backend.get_event(&contract_address, event_absolute_position)?;
        let (event, remainder) = T::from_bytes(&bytes).map_err(|_| EventError::Parsing)?;

        if remainder.is_empty() {
            Ok(event)
        } else {
            Err(EventError::Formatting)
        }
    }

    /// Retrieves a native event with the specified index from the specified contract.
    ///
    /// # Returns
    ///
    /// Returns the event as an instance of the specified type, or an error if the event
    /// couldn't be retrieved or parsed.
    pub fn get_native_event<T: FromBytes + EventInstance, R: Addressable>(
        &self,
        addr: &R,
        index: i32
    ) -> Result<T, EventError> {
        let contract_address = addr.address();
        let backend = self.backend.as_ref();
        let events_count = self.native_events_count(&contract_address);
        let event_absolute_position = crate::utils::event_absolute_position(events_count, index)
            .ok_or(EventError::IndexOutOfBounds)?;

        let bytes = backend.get_native_event(&contract_address, event_absolute_position)?;
        T::from_bytes(&bytes)
            .map_err(|_| EventError::Parsing)
            .map(|r| r.0)
    }

    /// Retrieves a raw event (serialized) with the specified index from the specified contract.
    pub fn get_event_bytes<T: Addressable>(
        &self,
        addr: &T,
        index: u32
    ) -> Result<Bytes, EventError> {
        let backend = self.backend.as_ref();
        backend.get_event(&addr.address(), index)
    }

    /// Retrieves a raw native event (serialized) with the specified index from the specified contract.
    pub fn get_native_event_bytes<T: Addressable>(
        &self,
        addr: &T,
        index: u32
    ) -> Result<Bytes, EventError> {
        let backend = self.backend.as_ref();
        backend.get_native_event(&addr.address(), index)
    }

    /// Returns the names of all events emitted by the specified contract.
    pub fn event_names<T: Addressable>(&self, addr: &T) -> Vec<String> {
        let events_count = self.events_count(addr);

        let backend = self.backend.as_ref();
        (0..events_count)
            .map(|event_id| {
                backend
                    .get_event(&addr.address(), event_id)
                    .and_then(|bytes| utils::extract_event_name(&bytes))
                    .unwrap_or_else(|e| panic!("Couldn't extract event name: {:?}", e))
            })
            .collect()
    }

    /// Returns all events emitted by the specified contract.
    pub fn events<T: Addressable>(&self, addr: &T) -> Vec<Bytes> {
        let backend = self.backend.as_ref();
        let contract_address = addr.address();
        let events_count = backend
            .get_events_count(&contract_address)
            .unwrap_or_default();
        (0..events_count)
            .map(|event_id| {
                backend
                    .get_event(&contract_address, event_id)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Couldn't get event at address {:?} with id {}: {:?}",
                            &contract_address, event_id, e
                        )
                    })
            })
            .collect()
    }

    /// Returns the number of events emitted by the specified contract.
    pub fn events_count<T: Addressable>(&self, addr: &T) -> u32 {
        let backend = self.backend.as_ref();
        backend
            .get_events_count(&addr.address())
            .unwrap_or_default()
    }

    /// Returns the number of native events emitted by the specified contract.
    pub fn native_events_count<T: Addressable>(&self, addr: &T) -> u32 {
        let backend = self.backend.as_ref();
        backend
            .get_native_events_count(&addr.address())
            .unwrap_or_default()
    }

    /// Returns true if the specified event was emitted by the specified contract.
    pub fn emitted_event<T: ToBytes + EventInstance, R: Addressable>(
        &self,
        addr: &R,
        event: T
    ) -> bool {
        let contract_address = addr.address();
        let events_count = self.events_count(addr);

        let event_bytes = Bytes::from(
            event
                .to_bytes()
                .unwrap_or_else(|_| panic!("Couldn't serialize event"))
        );

        (0..events_count)
            .map(|event_id| {
                self.get_event_bytes(&contract_address, event_id)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Couldn't get event at address {:?} with id {}: {:?}",
                            &contract_address, event_id, e
                        )
                    })
            })
            .any(|bytes| bytes == event_bytes)
    }

    /// Returns true if the specified event was emitted by the specified contract.
    pub fn emitted_native_event<T: ToBytes + EventInstance, R: Addressable>(
        &self,
        addr: &R,
        event: T
    ) -> bool {
        let contract_address = addr.address();
        let events_count = self.native_events_count(addr);
        if events_count > 0 {
            let event_bytes = Bytes::from(
                event
                    .to_bytes()
                    .unwrap_or_else(|_| panic!("Couldn't serialize event"))
            );
            (0..events_count)
                .map(|event_id| {
                    self.get_native_event_bytes(&contract_address, event_id)
                        .unwrap_or_else(|e| {
                            panic!(
                                "Couldn't get event at address {:?} with id {}: {:?}",
                                &contract_address, event_id, e
                            )
                        })
                })
                .any(|bytes| bytes == event_bytes)
        } else {
            false
        }
    }

    /// Returns true if an event with the specified name was emitted by the specified contract.
    pub fn emitted<T: AsRef<str>, R: Addressable>(&self, addr: &R, event_name: T) -> bool {
        let events_count = self.events_count(addr);

        (0..events_count)
            .map(|event_id| {
                self.get_event_bytes(addr, event_id).unwrap_or_else(|e| {
                    panic!(
                        "Couldn't get event at address {:?} with id {}: {:?}",
                        addr.address(),
                        event_id,
                        e
                    )
                })
            })
            .any(|bytes| {
                utils::extract_event_name(&bytes)
                    .unwrap_or_else(|e| panic!("Couldn't extract event name: {:?}", e))
                    .as_str()
                    == event_name.as_ref()
            })
    }
    /// Returns true if a native event with the specified name was emitted by the specified contract.
    pub fn emitted_native<T: AsRef<str>, R: Addressable>(&self, addr: &R, event_name: T) -> bool {
        let events_count = self.native_events_count(addr);

        (0..events_count)
            .map(|event_id| {
                self.get_native_event_bytes(addr, event_id)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Couldn't get event at address {:?} with id {}: {:?}",
                            addr.address(),
                            event_id,
                            e
                        )
                    })
            })
            .any(|bytes| {
                utils::extract_event_name(&bytes)
                    .unwrap_or_else(|e| panic!("Couldn't extract event name: {:?}", e))
                    .as_str()
                    == event_name.as_ref()
            })
    }

    pub(super) fn last_events(&self, contract: &mut DeployedContract) -> Vec<Bytes> {
        let old_count = contract.events_count;
        let new_count = self.events_count(&contract.address);
        let mut events = vec![];
        for count in old_count..new_count {
            let event = self.get_event_bytes(&contract.address, count).unwrap();
            events.push(event);
        }

        contract.events_count = new_count;
        events
    }

    pub(super) fn last_native_events(&self, contract: &mut DeployedContract) -> Vec<Bytes> {
        let old_count = contract.native_events_count;
        let new_count = self.native_events_count(&contract.address);
        let mut events = vec![];
        for count in old_count..new_count {
            let event = self
                .get_native_event_bytes(&contract.address, count)
                .unwrap();
            events.push(event);
        }

        contract.native_events_count = new_count;
        events
    }

    pub(super) fn init_events(&self, contract_address: &Address) {
        // First, check if initialization is needed and get event counts
        let needs_init = {
            let contracts = self.deployed_contracts.borrow();
            contracts
                .get(contract_address)
                .map(|contract| !contract.events_initialized)
                .unwrap_or(false)
        };

        if needs_init {
            // Get event counts while not holding any borrows
            let events_count = self.events_count(contract_address);
            let native_events_count = self.native_events_count(contract_address);

            // Now update the contract
            let mut contracts = self.deployed_contracts.borrow_mut();
            let contract = contracts.get_mut(contract_address).unwrap();
            contract.events_count = events_count;
            contract.native_events_count = native_events_count;
            contract.events_initialized = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::MockHostContext;
    use casper_event_standard::Event;
    use casper_types::account::AccountHash;
    use mockall::predicate;

    #[derive(Debug, Event, PartialEq)]
    struct TestEv {}

    #[test]
    fn test_get_event() {
        let addr = Address::Account(AccountHash::new([0; 32]));

        let mut ctx = MockHostContext::new();
        // there are 2 events emitted by the contract
        ctx.expect_get_events_count().returning(|_| Ok(2));
        // get_event() at index 0 will return an invalid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(0))
            .returning(|_, _| Ok(vec![1].into()));
        // get_event() at index 1 will return an valid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(1))
            .returning(|_, _| Ok(TestEv {}.to_bytes().unwrap().into()));

        let env = HostEnv::new(Rc::new(ctx));

        assert_eq!(env.get_event(&addr, 1), Ok(TestEv {}));
        assert_eq!(env.get_event(&addr, -1), Ok(TestEv {}));
        assert_eq!(
            env.get_event::<TestEv, _>(&addr, 0),
            Err(EventError::Parsing)
        );
        assert_eq!(
            env.get_event::<TestEv, _>(&addr, -2),
            Err(EventError::Parsing)
        );
        assert_eq!(
            env.get_event::<TestEv, _>(&addr, 2),
            Err(EventError::IndexOutOfBounds)
        );
        assert_eq!(
            env.get_event::<TestEv, _>(&addr, -3),
            Err(EventError::IndexOutOfBounds)
        );
    }

    #[test]
    fn test_events_works() {
        let addr = Address::Account(AccountHash::new([0; 32]));

        let mut ctx = MockHostContext::new();
        // there are 2 events emitted by the contract
        ctx.expect_get_events_count().returning(|_| Ok(2));
        // get_event() at index 0 will return an invalid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(0))
            .returning(|_, _| Ok(vec![1].into()));
        // get_event() at index 1 will return an valid event
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(1))
            .returning(|_, _| Ok(vec![1, 0, 1].into()));

        let env = HostEnv::new(Rc::new(ctx));

        assert_eq!(
            env.events(&addr),
            vec![vec![1].into(), vec![1, 0, 1].into()]
        );
    }

    #[test]
    #[should_panic(
        expected = "Couldn't get event at address Account(AccountHash(0000000000000000000000000000000000000000000000000000000000000000)) with id 0: CouldntExtractEventData"
    )]
    fn test_events_fails() {
        let addr = Address::Account(AccountHash::new([0; 32]));

        let mut ctx = MockHostContext::new();
        // there are 2 events emitted by the contract
        ctx.expect_get_events_count().returning(|_| Ok(2));
        // get_event() at index 0 panics
        ctx.expect_get_event()
            .with(predicate::always(), predicate::eq(0))
            .returning(|_, _| Err(EventError::CouldntExtractEventData));

        let env = HostEnv::new(Rc::new(ctx));

        env.events(&addr);
    }

    #[test]
    fn test_emitted() {
        let addr = Address::Account(AccountHash::new([0; 32]));
        let mut ctx = MockHostContext::new();

        ctx.expect_get_events_count().returning(|_| Ok(1));
        ctx.expect_get_event()
            .returning(|_, _| Ok(TestEv {}.to_bytes().unwrap().into()));

        let env = HostEnv::new(Rc::new(ctx));
        assert!(env.emitted(&addr, "TestEv"));
        assert!(!env.emitted(&addr, "AnotherEvent"));
    }
}
