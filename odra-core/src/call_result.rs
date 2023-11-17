use crate::HostEnv;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use casper_event_standard::EventInstance;
use odra_types::{Address, Bytes, OdraError, ToBytes};

#[derive(Debug, Clone)]
pub struct CallResult {
    pub contract_address: Address,
    pub caller: Address,
    pub gas_used: u64,
    pub result: Result<Bytes, OdraError>,
    pub events: BTreeMap<Address, Vec<Bytes>>
}

impl CallResult {
    pub fn result(&self) -> Bytes {
        match &self.result {
            Ok(result) => result.clone(),
            Err(error) => {
                panic!("Last call result is an error: {:?}", error);
            }
        }
    }

    pub fn error(&self) -> OdraError {
        match &self.result {
            Ok(_) => {
                panic!("Last call result is not an error");
            }
            Err(error) => error.clone()
        }
    }

    pub fn caller(&self) -> Address {
        self.caller
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn contract_address(&self) -> Address {
        self.contract_address
    }

    pub fn event_names(&self) -> Vec<String> {
        let mut event_names = vec![];
        self.events.values().for_each(|val| {
            val.iter().for_each(|bytes| {
                event_names.push(HostEnv::extract_event_name(bytes).unwrap());
            })
        });

        event_names
    }

    pub fn events(&self) -> Vec<Bytes> {
        let mut events = vec![];
        self.events.values().for_each(|val| {
            events.append(&mut val.clone());
        });

        events
    }

    pub fn contract_events(&self, contract_address: &Address) -> Vec<Bytes> {
        self.events.get(contract_address).unwrap_or(&vec![]).clone()
    }

    pub fn emitted(&self, event_name: &str) -> bool {
        self.event_names().contains(&event_name.to_string())
    }

    pub fn emitted_event<T: ToBytes + EventInstance>(&self, event: &T) -> bool {
        self.events()
            .contains(&Bytes::from(event.to_bytes().unwrap()))
    }
}
