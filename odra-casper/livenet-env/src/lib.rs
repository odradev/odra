pub mod env;

use env::LivenetEnv;
use odra_core::HostEnv;

pub fn livenet_env() -> HostEnv {
    let env = LivenetEnv::new();
    HostEnv::new(env)
}
