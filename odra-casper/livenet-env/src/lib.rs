//! This crate provides a host environment for the livenet.
pub mod livenet_contract_env;
pub mod livenet_host;
use livenet_host::LivenetHost;
use odra_core::host::HostEnv;

/// Returns a host environment for the livenet.
pub fn env() -> HostEnv {
    let env = LivenetHost::new();
    HostEnv::new(env)
}
