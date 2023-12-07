use crate::prelude::*;
use crate::utils::extract_event_name;
use crate::{Address, Bytes, OdraError, ToBytes};
use casper_event_standard::EventInstance;

#[derive(Debug, Clone)]
pub struct CallResult {
    pub(crate) contract_address: Address,
    pub(crate) caller: Address,
    pub(crate) gas_used: u64,
    pub(crate) result: Result<Bytes, OdraError>,
    pub(crate) events: BTreeMap<Address, Vec<Bytes>>
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

    pub fn event_names(&self, contract_address: &Address) -> Vec<String> {
        self.events
            .get(contract_address)
            .unwrap_or(&vec![])
            .iter()
            .map(|event_bytes| extract_event_name(event_bytes).unwrap())
            .collect()
    }

    pub fn contract_events(&self, contract_address: &Address) -> Vec<Bytes> {
        self.events.get(contract_address).unwrap_or(&vec![]).clone()
    }

    pub fn emitted(&self, contract_address: &Address, event_name: &str) -> bool {
        self.event_names(contract_address)
            .contains(&event_name.to_string())
    }

    pub fn emitted_event<T: ToBytes + EventInstance>(
        &self,
        contract_address: &Address,
        event: &T
    ) -> bool {
        self.contract_events(contract_address)
            .contains(&Bytes::from(event.to_bytes().unwrap()))
    }

    pub fn contract_last_call(self, contract_address: Address) -> ContractCallResult {
        ContractCallResult {
            call_result: self,
            contract_address
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContractCallResult {
    call_result: CallResult,
    contract_address: Address
}

impl ContractCallResult {
    pub fn new(call_result: CallResult, contract_address: Address) -> Self {
        Self {
            call_result,
            contract_address
        }
    }

    pub fn contract_address(&self) -> Address {
        self.contract_address
    }

    pub fn callee_contract_address(&self) -> Address {
        self.call_result.contract_address()
    }

    pub fn event_names(&self) -> Vec<String> {
        self.call_result.event_names(&self.contract_address)
    }

    pub fn events(&self) -> Vec<Bytes> {
        self.call_result.contract_events(&self.contract_address)
    }

    pub fn emitted(&self, event_name: &str) -> bool {
        self.call_result.emitted(&self.contract_address, event_name)
    }

    pub fn emitted_event<T: ToBytes + EventInstance>(&self, event: &T) -> bool {
        self.call_result
            .emitted_event(&self.contract_address, event)
    }
}
