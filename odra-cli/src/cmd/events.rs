use std::path::PathBuf;

use anyhow::Result;
use clap::ArgMatches;
use odra::{
    casper_types::bytesrepr::FromBytes, host::HostEnv, schema::casper_contract_schema::CustomType
};

use crate::{args, container, types, CustomTypeSet, DeployedContractsContainer, OdraCommand};

#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("The event structure for '{0}' is not valid")]
    InvalidEventSchema(String),
    #[error(transparent)]
    ArgsError(#[from] args::ArgsError),
    #[error(transparent)]
    TypesError(#[from] types::Error),
    #[error("Contract not found")]
    ContractNotFound,
    #[error("Event at {index} not found for contract '{contract_name}'")]
    EventNotFound { index: usize, contract_name: String },
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

    fn decode_event(&self, bytes: &[u8], event_name: &str, types: &CustomTypeSet) -> Result<()> {
        let ct = types
            .iter()
            .find(|ty| match ty {
                CustomType::Struct { name, .. } if name.0 == event_name => true,
                CustomType::Enum { name, .. } if name.0 == event_name => true,
                _ => false
            })
            .expect("Event type not found in custom types");
        println!("{}:", event_name);
        match ct {
            CustomType::Struct { members, .. } => {
                let (_name, rem): (String, _) = FromBytes::from_bytes(bytes)
                    .map_err(|_| EventError::InvalidEventSchema(event_name.to_string()))?;
                let mut bytes = rem;
                for m in members {
                    let (data, rem) = types::from_bytes(&m.ty.0, bytes)?;
                    bytes = rem;
                    println!("\t'{}': {}", m.name, data);
                }
            }
            CustomType::Enum { name, .. } => {
                return Err(EventError::InvalidEventSchema(name.0.clone()).into())
            }
        }
        Ok(())
    }
}

impl OdraCommand for PrintEventsCmd {
    fn name(&self) -> &str {
        &self.contract_name
    }

    fn run(
        &self,
        env: &HostEnv,
        _args: &ArgMatches,
        types: &CustomTypeSet,
        contracts_path: Option<PathBuf>
    ) -> Result<()> {
        let container = DeployedContractsContainer::load(contracts_path)?;
        let contract_address = container
            .address(&self.contract_name)
            .ok_or(EventError::ContractNotFound)?;
        let events = env.event_names(&contract_address);
        for (i, event_name) in events.iter().enumerate() {
            let ev = env
                .get_event_bytes(&contract_address, i as u32)
                .map_err(|_| EventError::EventNotFound {
                    index: i,
                    contract_name: self.contract_name.clone()
                })?;
            self.decode_event(&ev, event_name, types)?;
        }

        Ok(())
    }
}
