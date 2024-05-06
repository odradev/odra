use blake2::{digest::VariableOutput, Blake2bVar};
use odra::{
    casper_types::{BLAKE2B_DIGEST_LENGTH, U512},
    host::HostEnv,
    prelude::*,
    DeployReport
};
use std::io::Write;

pub const TEST_PRETTY_721_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": "abc",
  "token_uri": "https://www.barfoo.com"
}"#;
pub const TEST_PRETTY_UPDATED_721_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": "abc",
  "token_uri": "https://www.foobar.com"
}"#;
pub const TEST_PRETTY_CEP78_METADATA: &str = r#"{
  "name": "John Doe",
  "token_uri": "https://www.barfoo.com",
  "checksum": "940bffb3f2bba35f84313aa26da09ece3ad47045c6a1292c2bbd2df4ab1a55fb"
}"#;
pub const TEST_PRETTY_UPDATED_CEP78_METADATA: &str = r#"{
  "name": "John Doe",
  "token_uri": "https://www.foobar.com",
  "checksum": "fda4feaa137e83972db628e521c92159f5dc253da1565c9da697b8ad845a0788"
}"#;
pub const TEST_COMPACT_META_DATA: &str =
    r#"{"name": "John Doe","symbol": "abc","token_uri": "https://www.barfoo.com"}"#;
pub const MALFORMED_META_DATA: &str = r#"{
  "name": "John Doe",
  "symbol": abc,
  "token_uri": "https://www.barfoo.com"
}"#;

pub(crate) fn create_blake2b_hash<T: AsRef<[u8]>>(data: T) -> [u8; BLAKE2B_DIGEST_LENGTH] {
    let mut result = [0u8; 32];
    let mut hasher = <Blake2bVar as VariableOutput>::new(32).expect("should create hasher");
    let _ = hasher.write(data.as_ref());
    hasher
        .finalize_variable(&mut result)
        .expect("should copy hash to the result array");
    result
}

pub(crate) fn get_gas_cost_of(env: &HostEnv, entry_point: &str) -> Vec<U512> {
    let gas_report = env.gas_report();
    gas_report
        .into_iter()
        .filter_map(|r| match r {
            DeployReport::WasmDeploy { .. } => None,
            DeployReport::ContractCall { gas, call_def, .. } => {
                if call_def.entry_point() == entry_point {
                    Some(gas)
                } else {
                    None
                }
            }
        })
        .collect::<Vec<_>>()
}

pub(crate) fn get_deploy_gas_cost(env: &HostEnv) -> Vec<U512> {
    let gas_report = env.gas_report();
    gas_report
        .into_iter()
        .filter_map(|r| match r {
            DeployReport::WasmDeploy { gas, .. } => Some(gas),
            DeployReport::ContractCall { .. } => None
        })
        .collect::<Vec<_>>()
}
