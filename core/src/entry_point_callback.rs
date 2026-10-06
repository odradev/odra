//! A module that contains structs representing entry points and entry point callers.

use casper_types::CLType;

use crate::args::EntrypointArgument;
use crate::call_def::CallDef;
use crate::casper_types::bytesrepr::Bytes;
use crate::{host::HostEnv, prelude::*, ContractEnv};

/// A struct representing an entry point caller.
///
/// The caller is used by the host environment to call entry points of a contract.
///
/// This struct is responsible for calling the entry points of a contract.
/// It holds the host environment, a list of entry points, and a function pointer
/// that takes a contract environment and a call definition as arguments and returns
/// a result in the form of bytes.
#[derive(Clone)]
pub struct EntryPointsCaller {
    f: fn(contract_env: ContractEnv, call_def: CallDef) -> OdraResult<Bytes>,
    host_env: HostEnv,
    entry_points: Vec<EntryPoint>
}

impl EntryPointsCaller {
    /// Creates a new instance of `EntryPointsCaller`.
    ///
    /// # Arguments
    ///
    /// * `host_env` - The host environment.
    /// * `entry_points` - A collection of available entry points.
    /// * `f` - A function pointer that performs a call using a given contract environment and a call definition
    ///   and returns a result in the form of bytes.
    ///
    /// # Returns
    ///
    /// A new instance of `EntryPointsCaller`.
    pub fn new(
        host_env: HostEnv,
        entry_points: Vec<EntryPoint>,
        f: fn(contract_env: ContractEnv, call_def: CallDef) -> OdraResult<Bytes>
    ) -> Self {
        EntryPointsCaller {
            f,
            host_env,
            entry_points
        }
    }

    /// Calls the entry point with the given call definition.
    /// Returns the result of the entry point call in the form of bytes.
    pub fn call(&self, call_def: CallDef) -> OdraResult<Bytes> {
        (self.f)(self.host_env.contract_env(), call_def)
    }

    /// The function that runs an entry point in a given contract environment.
    pub fn callback(
        &self
    ) -> fn(contract_env: ContractEnv, call_def: CallDef) -> OdraResult<Bytes> {
        self.f
    }

    /// Returns a reference to the list of entry points.
    pub fn entry_points(&self) -> &[EntryPoint] {
        self.entry_points.as_ref()
    }

    /// Removes an entry point by its name.
    pub fn remove_entry_point(&mut self, name: &str) {
        self.entry_points.retain(|ep| ep.name != name);
    }
}

/// A struct representing an entry point.
#[derive(Clone)]
pub struct EntryPoint {
    /// The name of the entry point.
    pub name: String,
    /// The collection of arguments to the entry point.
    pub args: Vec<Argument>,
    /// A flag indicating whether the entry point is payable.
    pub is_payable: bool,
    /// A flag indicating that the function is `#[odra(offchain)]`: it is not deployed, the host
    /// runs it against the contract's state.
    pub is_offchain: bool
}

impl EntryPoint {
    /// Creates a new instance of `EntryPoint`.
    pub fn new(name: String, args: Vec<Argument>) -> Self {
        Self {
            name,
            args,
            is_payable: false,
            is_offchain: false
        }
    }

    /// Creates a new instance of payable `EntryPoint`.
    pub fn new_payable(name: String, args: Vec<Argument>) -> Self {
        Self {
            name,
            args,
            is_payable: true,
            is_offchain: false
        }
    }

    /// Creates a new instance of an offchain `EntryPoint`, see [EntryPoint::is_offchain].
    pub fn new_offchain(name: String, args: Vec<Argument>) -> Self {
        Self {
            name,
            args,
            is_payable: false,
            is_offchain: true
        }
    }
}

/// A struct representing an argument to entry point.
#[derive(Clone)]
pub struct Argument {
    /// The name of the argument.
    pub name: String,
    /// The type of the argument.
    pub ty: CLType
}

impl Argument {
    /// Creates a new instance of `Argument`.
    pub fn new<T: EntrypointArgument>(name: String) -> Self {
        Self {
            name,
            ty: T::cl_type()
        }
    }
}
