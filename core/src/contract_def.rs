//! Encapsulates a set of structures that abstract out a smart contract layout.

use crate::prelude::*;
use casper_event_standard::EventInstance;
use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    CLType, Key, PublicKey, URef, U128, U256, U512
};
use serde::{Deserialize, Serialize};

/// Contract's entrypoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entrypoint {
    /// The entrypoint's ident.
    pub name: String,
    /// The entrypoint's arguments.
    pub args: Vec<Argument>,
    /// `true` if the entrypoint is mutable.
    pub is_mutable: bool,
    /// The entrypoint's return type.
    pub return_ty: CLType,
    /// The entrypoint's type.
    pub ty: EntrypointType,
    /// The entrypoint's attributes.
    pub attributes: Vec<EntrypointAttribute>
}

/// Defines an argument passed to an entrypoint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Argument {
    /// The argument's ident.
    pub name: String,
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Event {
    /// The event's ident.
    pub name: String,
    /// The event's arguments.
    pub args: Vec<Argument>
}

impl Event {
    /// Returns `true` if the type of any argument is or contains `CLType::Any` (an
    /// `#[odra::odra_type]` struct or data-carrying enum, also inside an `Option`, a `Vec`, a map,
    /// a `Result` or a tuple). Such an event cannot be emitted: casper-event-standard rejects it.
    pub fn has_any(&self) -> bool {
        self.args.iter().any(|arg| cl_type_has_any(&arg.ty))
    }

    /// Checks that the event can be emitted, see [Event::has_any]. The error names the event,
    /// each offending field and what to do about it.
    pub fn validate(&self) -> Result<(), String> {
        let fields = self
            .args
            .iter()
            .filter(|arg| cl_type_has_any(&arg.ty))
            .map(|arg| format!("`{}`", arg.name))
            .collect::<Vec<_>>();
        if fields.is_empty() {
            return Ok(());
        }
        let (noun, verb) = match fields.len() {
            1 => ("field", "has"),
            _ => ("fields", "have")
        };
        Err(format!(
            "event `{}`: {} {} {} no concrete CLType (it is or contains an #[odra::odra_type] \
             struct or data-carrying enum, whose CLType is `Any`), so emitting it fails with \
             `Formatting`; casper-event-standard events need plain types: flatten the fields \
             into the event or use a unit enum",
            self.name,
            noun,
            fields.join(", "),
            verb
        ))
    }
}

/// Returns `true` if `ty` is or contains `CLType::Any`; the same check casper-event-standard
/// makes before it serializes an event field.
pub fn cl_type_has_any(ty: &CLType) -> bool {
    match ty {
        CLType::Any => true,
        CLType::Bool
        | CLType::I32
        | CLType::I64
        | CLType::U8
        | CLType::U32
        | CLType::U64
        | CLType::U128
        | CLType::U256
        | CLType::U512
        | CLType::Unit
        | CLType::String
        | CLType::Key
        | CLType::URef
        | CLType::PublicKey
        | CLType::ByteArray(_) => false,
        CLType::Option(ty) | CLType::List(ty) => cl_type_has_any(ty),
        CLType::Result { ok, err } => cl_type_has_any(ok) || cl_type_has_any(err),
        CLType::Map { key, value } => cl_type_has_any(key) || cl_type_has_any(value),
        CLType::Tuple1([ty]) => cl_type_has_any(ty),
        CLType::Tuple2([ty1, ty2]) => cl_type_has_any(ty1) || cl_type_has_any(ty2),
        CLType::Tuple3([ty1, ty2, ty3]) => {
            cl_type_has_any(ty1) || cl_type_has_any(ty2) || cl_type_has_any(ty3)
        }
    }
}

/// Panics if an event of contract `contract` cannot be emitted, see [Event::validate].
///
/// Called when a contract is deployed or upgraded on the host and when its schema is generated:
/// such an event compiles and deploys, and only its `emit_event` would revert, with a bare
/// `Formatting` error.
pub fn assert_events_can_be_emitted(contract: &str, events: &[Event]) {
    let errors = events
        .iter()
        .filter_map(|event| event.validate().err())
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!("Contract `{}`: {}", contract, errors.join("; "));
    }
}

/// Defines an entrypoint type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EntrypointType {
    /// A special entrypoint that can be called just once on the contract initialization.
    Constructor,
    /// A regular entrypoint.
    Public
}

/// Defines an entrypoint attribute.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EntrypointAttribute {
    /// A non-reentrant entrypoint.
    NonReentrant,
    /// A payable entrypoint.
    Payable
}

/// A trait that should be implemented by each smart contract to allow the backend.
pub trait HasIdent {
    /// Returns the contract's ident - the name of the module struct.
    fn ident() -> String;

    /// Returns the contract's name: the `name` given in `#[odra::module(name = "..")]`, or the
    /// ident when none was given. It names the contract in the schema and is the base of the
    /// named key the package hash is stored under (`<name>_package_hash`).
    ///
    /// Not called `name` on purpose: that is a common entry point name (token contracts).
    fn contract_name() -> String {
        Self::ident()
    }
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
    fn event_schemas() -> crate::prelude::BTreeMap<String, casper_event_standard::Schema> {
        crate::prelude::BTreeMap::new()
    }
}

/// Represents a contract blueprint.
///
/// A contract blueprint is a set of events and entrypoints defined in a smart contract.
/// It is used to generate the contract's ABI.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    ///
    /// # Panics
    ///
    /// If an event of the contract cannot be emitted, see [assert_events_can_be_emitted].
    pub fn new<T: HasIdent + HasEvents + HasEntrypoints>() -> Self {
        assert_events_can_be_emitted(&T::ident(), &T::events());
        Self {
            name: T::ident(),
            events: T::events(),
            entrypoints: T::entrypoints()
        }
    }

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
        let name = <T as EventInstance>::name();
        let schema = <T as EventInstance>::schema();
        let args = schema
            .to_vec()
            .iter()
            .map(|(name, ty)| Argument {
                name: name.clone(),
                ty: ty.clone().downcast(),
                is_ref: false,
                is_slice: false,
                is_required: true
            })
            .collect::<Vec<_>>();
        Event { name, args }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(name: &str, ty: CLType) -> Argument {
        Argument {
            name: String::from(name),
            ty,
            is_ref: false,
            is_slice: false,
            is_required: true
        }
    }

    fn event(args: Vec<Argument>) -> Event {
        Event {
            name: String::from("Traded"),
            args
        }
    }

    #[test]
    fn cl_type_has_any_looks_into_compound_types() {
        let any = || Box::new(CLType::Any);
        let u8 = || Box::new(CLType::U8);
        assert!(cl_type_has_any(&CLType::Any));
        assert!(cl_type_has_any(&CLType::Option(any())));
        assert!(cl_type_has_any(&CLType::List(any())));
        assert!(cl_type_has_any(&CLType::Map {
            key: CLType::String.into(),
            value: any()
        }));
        assert!(cl_type_has_any(&CLType::Map {
            key: any(),
            value: u8()
        }));
        assert!(cl_type_has_any(&CLType::Result {
            ok: u8(),
            err: any()
        }));
        assert!(cl_type_has_any(&CLType::Tuple1([any()])));
        assert!(cl_type_has_any(&CLType::Tuple2([u8(), any()])));
        assert!(cl_type_has_any(&CLType::Tuple3([u8(), u8(), any()])));
        assert!(cl_type_has_any(&CLType::Option(Box::new(CLType::List(
            Box::new(CLType::Tuple2([u8(), any()]))
        )))));

        assert!(!cl_type_has_any(&CLType::U8));
        assert!(!cl_type_has_any(&CLType::ByteArray(32)));
        assert!(!cl_type_has_any(&CLType::Option(Box::new(CLType::List(
            Box::new(CLType::Map {
                key: CLType::String.into(),
                value: CLType::Tuple3([u8(), CLType::Key.into(), CLType::U256.into()]).into()
            })
        )))));
    }

    #[test]
    fn an_event_with_any_cannot_be_emitted() {
        let ok = event(vec![arg("side", CLType::U8), arg("amount", CLType::U256)]);
        assert!(!ok.has_any());
        assert_eq!(ok.validate(), Ok(()));

        let one = event(vec![
            arg("side", CLType::U8),
            arg("price", CLType::Option(Box::new(CLType::Any))),
        ]);
        assert!(one.has_any());
        let err = one.validate().unwrap_err();
        assert!(
            err.starts_with("event `Traded`: field `price` has no concrete CLType"),
            "{err}"
        );
        assert!(err.contains("flatten the fields into the event or use a unit enum"));

        let two = event(vec![
            arg("price", CLType::Any),
            arg("prices", CLType::List(Box::new(CLType::Any))),
        ]);
        assert!(two
            .validate()
            .unwrap_err()
            .starts_with("event `Traded`: fields `price`, `prices` have no concrete CLType"));
    }

    struct Exchange;

    impl HasIdent for Exchange {
        fn ident() -> String {
            String::from("Exchange")
        }
    }

    impl HasEvents for Exchange {
        fn events() -> Vec<Event> {
            vec![event(vec![arg("price", CLType::Any)])]
        }
    }

    impl HasEntrypoints for Exchange {
        fn entrypoints() -> Vec<Entrypoint> {
            vec![]
        }
    }

    #[test]
    #[should_panic(
        expected = "Contract `Exchange`: event `Traded`: field `price` has no concrete CLType"
    )]
    fn the_schema_of_a_contract_with_such_an_event_is_not_generated() {
        ContractBlueprint::new::<Exchange>();
    }
}
