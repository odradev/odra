use crate::address::Address;

#[derive(Debug, Clone)]
pub struct DeployedContract {
    pub address: Address,
    pub current_version: u32,
    pub events_count: u32,
    pub native_events_count: u32,
    pub events_initialized: bool
}

impl DeployedContract {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            current_version: 0,
            events_count: 0,
            native_events_count: 0,
            events_initialized: false
        }
    }
}
