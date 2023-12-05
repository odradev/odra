//! Encapsulates a set of structures that abstract out a smart contract layout.

use crate::{prelude::*, Address};
use casper_event_standard::EventInstance;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLType, Key, PublicKey, URef, U128, U256, U512
};

/// Contract's entrypoint.
#[derive(Debug, Clone)]
pub struct Entrypoint {
    pub ident: String,
    pub args: Vec<Argument>,
    pub is_mut: bool,
    pub ret: CLType,
    pub ty: EntrypointType
}

/// Defines an argument passed to an entrypoint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Argument {
    pub ident: String,
    pub ty: CLType,
    pub is_ref: bool,
    pub is_slice: bool
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Event {
    pub ident: String,
    pub args: Vec<Argument>
}

impl Event {
    pub fn has_any(&self) -> bool {
        self.args.iter().any(|arg| arg.ty == CLType::Any)
    }
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

#[derive(Debug, Clone)]
pub struct ContractBlueprint {
    pub name: &'static str,
    pub events: Vec<Event>,
    pub entrypoints: Vec<Entrypoint>
}

pub trait IntoEvent {
    fn into_event() -> Event;
}

impl<T: EventInstance> IntoEvent for T {
    fn into_event() -> Event {
        let ident = <T as EventInstance>::name();
        let schema = <T as EventInstance>::schema();
        let args = schema
            .to_vec()
            .iter()
            .map(|(name, ty)| Argument {
                ident: name.clone(),
                ty: ty.clone().downcast(),
                is_ref: false,
                is_slice: false
            })
            .collect::<Vec<_>>();
        Event { ident, args }
    }
}

macro_rules! impl_has_events {
    ($($t:ty),*) => {
        impl HasEvents for () {
            fn events() -> Vec<Event> {
                vec![]
            }
        }

        $(
            impl HasEvents for $t {
                fn events() -> Vec<Event> {
                    vec![]
                }
            }
        )*
    };
}

impl_has_events!(
    u8, u16, u32, u64, i8, i16, i32, i64, U128, U256, U512, Address, String, bool, Key, URef,
    PublicKey
);

impl<T: ToBytes + FromBytes> HasEvents for Option<T> {
    fn events() -> Vec<Event> {
        vec![]
    }
}

impl<T: ToBytes + FromBytes, E: ToBytes + FromBytes> HasEvents for Result<T, E> {
    fn events() -> Vec<Event> {
        vec![]
    }
}

impl<T: ToBytes + FromBytes, E: ToBytes + FromBytes> HasEvents for BTreeMap<T, E> {
    fn events() -> Vec<Event> {
        vec![]
    }
}

impl<T: ToBytes + FromBytes> HasEvents for Vec<T> {
    fn events() -> Vec<Event> {
        vec![]
    }
}

impl<T1: ToBytes + FromBytes> HasEvents for (T1,) {
    fn events() -> Vec<Event> {
        vec![]
    }
}

impl<T1: ToBytes + FromBytes, T2: ToBytes + FromBytes> HasEvents for (T1, T2) {
    fn events() -> Vec<Event> {
        vec![]
    }
}

impl<T1: ToBytes + FromBytes, T2: ToBytes + FromBytes, T3: ToBytes + FromBytes> HasEvents
    for (T1, T2, T3)
{
    fn events() -> Vec<Event> {
        vec![]
    }
}
