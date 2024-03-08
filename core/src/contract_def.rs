//! Encapsulates a set of structures that abstract out a smart contract layout.

use crate::{prelude::*, Address};
use casper_event_standard::EventInstance;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLType, Key, PublicKey, URef, U128, U256, U512
};
#[cfg(not(target_arch = "wasm32"))]
use serde::{Deserialize, Serialize};

/// Contract's entrypoint.
#[derive(Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct Entrypoint {
    /// The entrypoint's ident.
    pub ident: String,
    /// The entrypoint's arguments.
    pub args: Vec<Argument>,
    /// `true` if the entrypoint is mutable.
    pub is_mut: bool,
    /// The entrypoint's return type.
    pub ret: CLType,
    /// The entrypoint's type.
    pub ty: EntrypointType,
    /// The entrypoint's attributes.
    pub attributes: Vec<EntrypointAttribute>
}

/// Defines an argument passed to an entrypoint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct Argument {
    /// The argument's ident.
    pub ident: String,
    /// The argument's type.
    pub ty: CLType,
    /// `true` if the argument is a reference.
    pub is_ref: bool,
    /// `true` if the argument is a slice.
    pub is_slice: bool,
    /// `true` if the argument is required.
    pub is_required: bool
}

/// Defines an event.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct Event {
    /// The event's ident.
    pub ident: String,
    /// The event's arguments.
    pub args: Vec<Argument>
}

impl Event {
    /// Returns `true` if the event has any argument of `CLType::Any` type.
    pub fn has_any(&self) -> bool {
        self.args.iter().any(|arg| arg.ty == CLType::Any)
    }
}

/// Defines an entrypoint type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub enum EntrypointType {
    /// A special entrypoint that can be called just once on the contract initialization.
    Constructor,
    /// A regular entrypoint.
    Public
}

/// Defines an entrypoint attribute.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub enum EntrypointAttribute {
    /// A non-reentrant entrypoint.
    NonReentrant,
    /// A payable entrypoint.
    Payable
}

/// A trait that should be implemented by each smart contract to allow the backend.
pub trait HasIdent {
    /// Returns the contract's ident.
    fn ident() -> String;
}

/// A trait that should be implemented by each smart contract to allow the backend
/// to generate blockchain-specific code.
pub trait HasEntrypoints {
    /// Returns the list of contract's entrypoints.
    fn entrypoints() -> Vec<Entrypoint>;
}

/// A trait that should be implemented by each smart contract to allow the backend.
pub trait HasEvents {
    /// Returns a list of Events used by the contract.
    fn events() -> Vec<Event>;

    /// Returns a map of event schemas used by the contract.
    #[cfg(target_arch = "wasm32")]
    fn event_schemas() -> crate::prelude::BTreeMap<String, casper_event_standard::Schema> {
        crate::prelude::BTreeMap::new()
    }
}

/// Represents a contract blueprint.
///
/// A contract blueprint is a set of events and entrypoints defined in a smart contract.
/// It is used to generate the contract's ABI.
#[derive(Debug, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Serialize, Deserialize))]
pub struct ContractBlueprint {
    /// The name of the contract.
    pub name: String,
    /// The events defined in the contract.
    pub events: Vec<Event>,
    /// The entrypoints defined in the contract.
    pub entrypoints: Vec<Entrypoint>
}

impl ContractBlueprint {
    /// Creates a new instance of `ContractBlueprint` using the provided type parameters.
    ///
    /// # Type Parameters
    ///
    /// - `T`: A type that implements the `HasIdent`, `HasEvents`, and `HasEntrypoints` traits.
    ///
    /// # Returns
    ///
    /// A new instance of `ContractBlueprint` with the name, events, and entrypoints
    /// obtained from the type `T`.
    pub fn new<T: HasIdent + HasEvents + HasEntrypoints>() -> Self {
        Self {
            name: T::ident(),
            events: T::events(),
            entrypoints: T::entrypoints()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Converts the `ContractBlueprint` instance to a JSON string representation.
    ///
    /// # Returns
    ///
    /// A `Result` containing the JSON string if the conversion is successful,
    /// or a `serde_json::Error` if an error occurs during serialization.
    pub fn as_json(self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }
}

/// A trait for converting a type into an [Event].
pub trait IntoEvent {
    /// Converts the type into an [Event].
    fn into_event() -> Event;
}

impl<T: EventInstance> IntoEvent for T {
    fn into_event() -> Event {
        let mut schemas = casper_event_standard::Schemas::new();
        schemas.add::<T>();
        let ident = <T as EventInstance>::name();
        let schema = <T as EventInstance>::schema();
        let args = schema
            .to_vec()
            .iter()
            .map(|(name, ty)| Argument {
                ident: name.clone(),
                ty: ty.clone().downcast(),
                is_ref: false,
                is_slice: false,
                is_required: true
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

impl<T: ToBytes + FromBytes, const N: usize> HasEvents for [T; N] {
    fn events() -> Vec<Event> {
        vec![]
    }
}

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
