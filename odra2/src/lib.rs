#![no_std]

pub use odra_core::{
    mapping::Mapping, module, module::ModuleWrapper, prelude, variable::Variable, CallDef,
    ContractEnv, HostEnv
};
pub use odra_types as types;

#[cfg(target_arch = "wasm32")]
pub use odra_casper_backend2;

mod casper_test {
    use odra_casper_test_env2;
    use odra_core::HostContext;

    pub fn test_env() -> odra_core::HostEnv {
        odra_casper_test_env2::CasperVm::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "casper")]
pub use casper_test::*;

// mod odra_vm_test {
//     use odra_vm_test_env;
//     use odra_core::HostContext;
//
//     pub fn test_env() -> odra_core::HostEnv {
//         odra_vm_test_env::OdraVm::new()
//     }
// }
//
// #[cfg(feature = "odra-vm")]
// pub use odra_vm_test::*;
