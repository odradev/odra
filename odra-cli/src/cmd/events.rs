use std::path::PathBuf;

use anyhow::Result;
use clap::ArgMatches;
use odra::host::HostEnv;

use crate::{args, container, types, CustomTypeSet, DeployedContractsContainer, OdraCommand};

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error(transparent)]
    ArgsError(#[from] args::ArgsError),
    #[error(transparent)]
    TypesError(#[from] types::Error),
    #[error("Contract not found")]
    ContractNotFound,
    #[error("Event at {index} not found for contract '{contract_name}'")]
    EventNotFound { index: u32, contract_name: String },
    #[error(transparent)]
    ContractError(#[from] container::ContractError)
}

pub(crate) struct PrintEventsCmd {
    contract_name: String
}

impl PrintEventsCmd {
    pub fn new(contract_name: String) -> Self {
        PrintEventsCmd { contract_name }
    }
}

impl OdraCommand for PrintEventsCmd {
    fn name(&self) -> &str {
        &self.contract_name
    }

    fn run(
        &self,
        env: &HostEnv,
        args: &ArgMatches,
        types: &CustomTypeSet,
        contracts_path: Option<PathBuf>
    ) -> Result<()> {
        let container = DeployedContractsContainer::load(contracts_path)?;
        let contract_address = container
            .address(&self.contract_name)
            .ok_or(EventError::ContractNotFound)?;
        let events_count = env.events_count(&contract_address);
        let max_events = args::read(args, "n", |s| s.parse()).unwrap_or(events_count);
        let max_events = max_events.min(events_count);
        prettycli::info(&format!(
            "Printing {:?} the most recent events for contract '{}'",
            max_events, self.contract_name
        ));
        for i in 0..max_events {
            let idx = events_count - i - 1;
            let ev = env.get_event_bytes(&contract_address, idx).map_err(|_| {
                EventError::EventNotFound {
                    index: i,
                    contract_name: self.contract_name.clone()
                }
            })?;

            prettycli::info(&format!(
                "Event {}: {}",
                i + 1,
                types::decode_event(&ev, types)?
            ));
        }

        Ok(())
    }
}
