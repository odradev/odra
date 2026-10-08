//! A field may have any name, also the name of a variable of the code that Odra generates for a
//! module, a type or an event (`env`, `result`, `bytes`, ...): the locals of the generated code are
//! prefixed, so they never clash with the fields.
use odra::casper_types::{bytesrepr::Bytes, U256};
use odra::prelude::*;

/// A type whose fields are named like the internals of the generated (de)serialization code.
#[odra::odra_type]
pub struct NamedFields {
    /// A number.
    pub result: u32,
    /// A text.
    pub bytes: String,
    /// An optional value.
    pub value: Option<u8>
}

/// An enum whose variant fields are named like the internals of the generated code.
#[odra::odra_type]
pub enum NamedVariants {
    /// Fields named like the generated locals.
    Named {
        /// A number.
        result: u32,
        /// Raw bytes.
        bytes: Bytes
    },
    /// Unnamed fields.
    Tuple(u32, String),
    /// No fields.
    Unit
}

/// An event whose fields are named like the internals of the generated code.
#[odra::event]
pub struct FieldsSet {
    /// The previous value.
    pub bytes: u32,
    /// The new value.
    pub result: U256,
    /// Who set it.
    pub env: Address
}

/// A module whose fields are named like the internals of the generated code.
#[odra::module(events = [FieldsSet])]
pub struct FieldNames {
    env: Var<u32>,
    result: Var<NamedFields>,
    bytes: Mapping<u32, NamedVariants>
}

#[odra::module]
impl FieldNames {
    /// Stores the arguments.
    pub fn init(&mut self, env: u32, result: NamedFields) {
        self.env.set(env);
        self.result.set(result);
    }

    /// Replaces the number and emits [FieldsSet].
    pub fn set_env(&mut self, value: u32) {
        let bytes = self.env.get_or_default();
        self.env.set(value);
        self.env().emit_event(FieldsSet {
            bytes,
            result: U256::from(value),
            env: self.env().caller()
        });
    }

    /// The number.
    pub fn env_value(&self) -> u32 {
        self.env.get_or_default()
    }

    /// The stored type.
    pub fn result(&self) -> Option<NamedFields> {
        self.result.get()
    }

    /// Stores a variant under `key`.
    pub fn set_bytes(&mut self, key: u32, value: NamedVariants) {
        self.bytes.set(&key, value);
    }

    /// The variant under `key`.
    pub fn bytes(&self, key: u32) -> Option<NamedVariants> {
        self.bytes.get(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::{FieldNames, FieldNamesInitArgs, FieldsSet, NamedFields, NamedVariants};
    use odra::casper_event_standard::EventInstance;
    use odra::casper_types::bytesrepr::{FromBytes, ToBytes};
    use odra::casper_types::{bytesrepr::Bytes, U256};
    use odra::host::Deployer;
    use odra::prelude::*;

    fn named_fields() -> NamedFields {
        NamedFields {
            result: 7,
            bytes: String::from("ab"),
            value: Some(3)
        }
    }

    #[test]
    fn fields_named_like_the_generated_code_work() {
        let env = odra_test::env();
        let mut contract = FieldNames::deploy(
            &env,
            FieldNamesInitArgs {
                env: 1,
                result: named_fields()
            }
        );
        assert_eq!(contract.env_value(), 1);
        assert_eq!(contract.result(), Some(named_fields()));

        contract.set_env(5);
        assert_eq!(contract.env_value(), 5);
        assert!(env.emitted_event(
            &contract,
            FieldsSet {
                bytes: 1,
                result: U256::from(5),
                env: env.get_account(0)
            }
        ));

        let variants = [
            NamedVariants::Named {
                result: 9,
                bytes: Bytes::from(vec![1, 2])
            },
            NamedVariants::Tuple(4, String::from("x")),
            NamedVariants::Unit
        ];
        for (key, variant) in variants.iter().enumerate() {
            contract.set_bytes(key as u32, variant.clone());
        }
        for (key, variant) in variants.into_iter().enumerate() {
            assert_eq!(contract.bytes(key as u32), Some(variant));
        }
    }

    #[test]
    fn the_serialized_layout_does_not_depend_on_field_names() {
        // A struct is its fields in order, an enum is its index followed by the fields.
        let value = named_fields();
        let expected = [
            7u32.to_bytes().unwrap(),
            "ab".to_bytes().unwrap(),
            Some(3u8).to_bytes().unwrap()
        ]
        .concat();
        assert_eq!(value.to_bytes().unwrap(), expected);
        assert_eq!(value.serialized_length(), expected.len());
        assert_eq!(
            NamedFields::from_bytes(&expected).unwrap(),
            (value, &[][..])
        );

        let variant = NamedVariants::Named {
            result: 9,
            bytes: Bytes::from(vec![1, 2])
        };
        let expected = [
            vec![0u8],
            9u32.to_bytes().unwrap(),
            Bytes::from(vec![1u8, 2]).to_bytes().unwrap()
        ]
        .concat();
        assert_eq!(variant.to_bytes().unwrap(), expected);
        assert_eq!(variant.serialized_length(), expected.len());
        assert_eq!(
            NamedVariants::from_bytes(&expected).unwrap(),
            (variant, &[][..])
        );
        let expected = [vec![1u8], 4u32.to_bytes().unwrap(), "x".to_bytes().unwrap()].concat();
        assert_eq!(
            NamedVariants::Tuple(4, String::from("x"))
                .to_bytes()
                .unwrap(),
            expected
        );
        assert_eq!(NamedVariants::Unit.to_bytes().unwrap(), vec![2u8]);
    }

    /// The same event twice: generated by `#[odra::event]` and by `casper_event_standard`'s own
    /// derive (which does not compile with a field named `bytes`).
    mod ours {
        use odra::casper_types::{bytesrepr::Bytes, U256};
        use odra::prelude::*;

        #[odra::event]
        pub struct Transfer {
            pub amount: U256,
            pub result: Option<String>,
            pub env: Address,
            pub value: Bytes
        }
    }

    mod derived {
        use odra::casper_types::{bytesrepr::Bytes, U256};
        use odra::prelude::*;

        #[derive(odra::Event, PartialEq, Eq, Debug)]
        pub struct Transfer {
            pub amount: U256,
            pub result: Option<String>,
            pub env: Address,
            pub value: Bytes
        }
    }

    #[test]
    fn an_event_serializes_like_the_casper_event_standard_derive() {
        let env = odra_test::env();
        let ours = ours::Transfer {
            amount: U256::from(500),
            result: Some(String::from("ok")),
            env: env.get_account(1),
            value: Bytes::from(vec![1, 2, 3])
        };
        let derived = derived::Transfer {
            amount: U256::from(500),
            result: Some(String::from("ok")),
            env: env.get_account(1),
            value: Bytes::from(vec![1, 2, 3])
        };
        let bytes = ours.to_bytes().unwrap();
        assert_eq!(bytes, derived.to_bytes().unwrap());
        assert_eq!(ours.serialized_length(), derived.serialized_length());
        assert_eq!(ours::Transfer::from_bytes(&bytes).unwrap(), (ours, &[][..]));
        assert_eq!(ours::Transfer::name(), derived::Transfer::name());
        assert_eq!(
            ours::Transfer::schema().to_bytes().unwrap(),
            derived::Transfer::schema().to_bytes().unwrap()
        );

        let event = FieldsSet {
            bytes: 1,
            result: U256::from(5),
            env: env.get_account(1)
        };
        let bytes = event.to_bytes().unwrap();
        assert_eq!(bytes.len(), event.serialized_length());
        assert_eq!(FieldsSet::from_bytes(&bytes).unwrap(), (event, &[][..]));
    }
}
