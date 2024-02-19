#![doc = "OdraVM is a mock VM for testing contracts written id Odra Framework."]
#![doc = "It is a simple implementation of a mock backend with a minimal set of features"]
#![doc = "that allows testing the code written in Odra without compiling the contract"]
#![doc = "to the target architecture and spinning up the blockchain."]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod odra_vm_contract_env;
mod odra_vm_host;
mod vm;

pub use odra_vm_host::OdraVmHost;
pub use vm::OdraVm;
