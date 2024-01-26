//! This crate provides a host environment for the livenet.
#![feature(once_cell)]
pub mod livenet_contract_env;
pub mod livenet_host;
use livenet_host::LivenetHost;
use odra_core::HostEnv;

/// Returns a host environment for the livenet.
pub fn env() -> HostEnv {
    let env = LivenetHost::new();
    HostEnv::new(env)
}
