//! Encapsulates a set of structures that abstract out a smart contract layout.
use odra_types::CLType;

/// Top-level contract abstraction.
#[derive(Debug)]
pub struct ContractDef {
    pub ident: String,
    pub entrypoints: Vec<Entrypoint>,
}

/// Contract's entrypoint.
#[derive(Debug)]
pub struct Entrypoint {
    pub ident: String,
    pub args: Vec<Argument>,
    pub ret: CLType,
    pub ty: EntrypointType,
}

/// Defines an argument passed to an entrypoint.
#[derive(Debug)]
pub struct Argument {
    pub ident: String,
    pub ty: CLType,
}

/// Defines an entrypoint type.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntrypointType {
    /// A special entrypoint that can be called just once on the contract initialization.
    Constructor,
    /// A regular entrypoint.
    Public,
}

/// A trait that should be implemented by each smart contract to allow the backend
/// to generate blockchain-specific code.
///
/// Probably you will never implement this trait by your own, it is automatically
/// implemented by [odra::module](crate::module) macro.
pub trait HasContractDef {
    /// Returns an abstract contract definition.
    fn contract_def() -> ContractDef;
}
