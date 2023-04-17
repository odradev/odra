pub mod casper_client;
pub mod client_env;
pub mod contract_env;

use odra_casper_types::CallArgs;

pub type EntrypointCall = fn(String, CallArgs) -> Vec<u8>;
pub type EntrypointArgs = Vec<String>;
