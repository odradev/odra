use crate::address::Address;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct DeployedContract {
    pub address: Address,
    /// The name of the contract code the backend was last given for the address (deploy,
    /// upgrade or registration), `None` for a contract deployed by a factory.
    pub name: Option<String>,
    pub current_version: u32,
    pub events_count: u32,
    pub native_events_count: u32,
    pub events_initialized: bool
}

impl DeployedContract {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            name: None,
            current_version: 0,
            events_count: 0,
            native_events_count: 0,
            events_initialized: false
        }
    }

    pub fn with_name(address: Address, name: &str) -> Self {
        Self {
            name: Some(String::from(name)),
            ..Self::new(address)
        }
    }
}
