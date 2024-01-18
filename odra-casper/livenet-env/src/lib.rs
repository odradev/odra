#![feature(once_cell)]
pub mod livenet_contract_env;
pub mod livenet_host_env;
use livenet_host_env::LivenetEnv;
use odra_core::HostEnv;

pub fn livenet_env() -> HostEnv {
    let env = LivenetEnv::new();
    HostEnv::new(env)
}
