use casper_event_standard::EventInstance;
use casper_types::{bytesrepr::FromBytes, RuntimeArgs, U512};

use crate::{contract_def::HasIdent, entry_point_callback::EntryPointsCaller, Address, ContractCallResult, EventError, HostEnv};

pub trait HostRef {
    fn new(address: Address, env: HostEnv) -> Self;
    fn with_tokens(&self, tokens: U512) -> Self;
    fn address(&self) -> &Address;
    fn env(&self) -> &HostEnv;
    fn get_event<T>(&self, index: i32) -> Result<T, EventError> 
        where T: FromBytes + EventInstance;
    fn last_call(&self) -> ContractCallResult;
}

pub trait HostRefLoader {
    fn load(env: &HostEnv, address: Address) -> Self;
}

pub trait EntryPointCallerProvider {
    fn epc(env: &HostEnv) -> EntryPointsCaller;
}

/// Trait for deployable contracts
pub trait Deployer {
    /// Deploy a contract with given init args
    fn deploy<T: Into<Option<RuntimeArgs>>>(env: &HostEnv, init_args: T) -> Self;
}

impl <R: HostRef + EntryPointCallerProvider + HasIdent> Deployer for R {
    fn deploy<T: Into<Option<RuntimeArgs>>>(env: &HostEnv, init_args: T) -> Self {
        let caller = R::epc(env);
        let address = env.new_contract(
            &R::ident(),
            init_args.into(),
            caller,
        );
        R::new(address, env.clone())
    }
}

impl<T: EntryPointCallerProvider + HostRef> HostRefLoader for T {
    fn load(env: &HostEnv, address: Address) -> Self {
        let caller = T::epc(env);
        env.register_contract(address, caller);
        T::new(address, env.clone())
    }
}
