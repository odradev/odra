//! Encapsulates a set of structures that abstract out a smart contract layout.

use alloc::{string::String, vec::Vec};

use crate::Type;

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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Argument {
    pub ident: String,
    pub ty: Type,
    pub is_ref: bool
}

/// Defines an entrypoint type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntrypointType {
    /// A special entrypoint that can be called just once on the contract initialization.
    Constructor { non_reentrant: bool },
    /// A regular entrypoint.
    Public { non_reentrant: bool },
    /// A payable entrypoint.
    PublicPayable { non_reentrant: bool }
}

impl EntrypointType {
    pub fn is_non_reentrant(&self) -> bool {
        match self {
            EntrypointType::Constructor { non_reentrant } => *non_reentrant,
            EntrypointType::Public { non_reentrant } => *non_reentrant,
            EntrypointType::PublicPayable { non_reentrant } => *non_reentrant
        }
    }
}

/// Defines an event.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    pub ident: String,
    pub args: Vec<Argument>
}

/// A trait that should be implemented by each smart contract to allow the backend
/// to generate blockchain-specific code.
pub trait HasEntrypoints {
    /// Returns an abstract contract definition.
    fn entrypoints() -> Vec<Entrypoint>;
}

/// A trait that should be implemented by each smart contract to allow the backend.
pub trait HasIdent {
    fn ident() -> String;
}
/// A trait that should be implemented by each smart contract to allow the backend.
pub trait HasEvents {
    fn events() -> Vec<Event>;
}
