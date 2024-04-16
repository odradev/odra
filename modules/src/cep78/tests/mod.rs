use odra::{
    args::Maybe,
    host::{Deployer, HostRef}
};

mod acl;
mod set_variables;
mod utils;

use crate::cep78::{error::CEP78Error, events::VariablesSet, modalities::OwnerReverseLookupMode};

use super::token::CEP78HostRef;

pub(super) const COLLECTION_NAME: &str = "CEP78-Test-Collection";
pub(super) const COLLECTION_SYMBOL: &str = "CEP78";
pub(super) const TOTAL_TOKEN_SUPPLY: u64 = 100_000_000;
