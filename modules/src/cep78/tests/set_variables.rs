use odra::{
    args::Maybe,
    host::{Deployer, HostRef}
};

use super::{COLLECTION_NAME, COLLECTION_SYMBOL};
use crate::cep78::{error::CEP78Error, events::VariablesSet, token::CEP78HostRef};
