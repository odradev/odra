//! A module that contains structs representing entry points and entry point callers.

use casper_types::CLType;

use crate::call_def::CallDef;
use crate::casper_types::bytesrepr::Bytes;
use crate::{prelude::*, ContractEnv, HostEnv, OdraResult};

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
    ///         and returns a result in the form of bytes.
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

    /// Returns a reference to the list of entry points.
    pub fn entry_points(&self) -> &[EntryPoint] {
        self.entry_points.as_ref()
    }
}

/// A struct representing an entry point.
#[derive(Clone)]
pub struct EntryPoint {
    /// The name of the entry point.
    pub name: String,
    /// The collection of arguments to the entry point.
    pub args: Vec<Argument>
}

impl EntryPoint {
    /// Creates a new instance of `EntryPoint`.
    pub fn new(name: String, args: Vec<Argument>) -> Self {
        Self { name, args }
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
    pub fn new(name: String, ty: CLType) -> Self {
        Self { name, ty }
    }
}
