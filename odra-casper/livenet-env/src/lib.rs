//! This crate provides a host environment for the livenet.
mod error;
mod livenet_contract_env;
mod livenet_host;

use livenet_host::LivenetHost;
pub use odra_casper_rpc_client::error::LivenetError;
use odra_core::host::HostEnv;

/// Returns a host environment for the livenet.
pub fn env() -> HostEnv {
    let env = LivenetHost::new();
    HostEnv::new(env)
}

/// Returns a non-failing host env for the livent
pub fn env_safe() -> Result<HostEnv, LivenetError> {
    let env = LivenetHost::new_safe()?;
    Ok(HostEnv::new(env))
}
