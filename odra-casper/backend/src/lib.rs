//! Generate Casper contract and interact with Casper host.

mod casper_env;
pub mod contract_env;
pub mod utils;

pub use casper_contract::{
    self,
    contract_api::{runtime, storage}
};
