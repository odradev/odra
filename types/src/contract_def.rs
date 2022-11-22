//! Encapsulates a set of structures that abstract out a smart contract layout.

use crate::Type;

/// Top-level contract abstraction.
#[derive(Debug, Clone)]
pub struct ContractDef {
    pub ident: String,
    pub entrypoints: Vec<Entrypoint>
}

/// Contract's entrypoint.
#[derive(Debug, Clone)]
pub struct Entrypoint {
    pub ident: String,
    pub args: Vec<Argument>,
    pub is_mut: bool,
    pub ret: Type,
    pub ty: EntrypointType
}

/// Defines an argument passed to an entrypoint.
#[derive(Debug, Clone)]
pub struct Argument {
    pub ident: String,
    pub ty: Type
}

/// Defines an entrypoint type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntrypointType {
    /// A special entrypoint that can be called just once on the contract initialization.
    Constructor,
    /// A regular entrypoint.
    Public,
    /// A payable entrypoint.
    PublicPayable
}

/// A trait that should be implemented by each smart contract to allow the backend
/// to generate blockchain-specific code.
pub trait HasContractDef {
    /// Returns an abstract contract definition.
    fn contract_def() -> ContractDef;
}
