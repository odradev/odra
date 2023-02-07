use std::collections::BTreeMap;

use odra_types::Type;
use serde_json::Value;

use crate::{Typed, CosmosType, Address, U256, U512};

/// Represents a collection of arguments passed to a smart contract entrypoint call.
#[derive(Default, Debug)]
pub struct CallArgs {
    data: BTreeMap<String, (Vec<u8>, Type)>
}

impl CallArgs {
    /// Creates a new no-args instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a new empty arg into the collection.
    pub fn insert<K: Into<String>, V: CosmosType + Typed>(&mut self, key: K, value: V) {
        // TODO: Handle unwrap.
        self.data.insert(key.into(), (value.ser().unwrap(), V::ty()));
    }

    /// Gets an argument by the name.
    pub fn get(&self, key: &str) -> (Vec<u8>, Type) {
        // TODO: Handle unwraps.
        self.data.get(key).cloned().unwrap()
    }

    pub fn get_as_value(&self, key: &str) -> Value {
        dbg!(key);
        let (value, ty) = self.get(key);
        Self::parse_value(key, value, ty)
        
    }

    fn parse_value(key: &str, value: Vec<u8>, ty: Type) -> Value {
        dbg!(&value);
        dbg!(&ty);
        match ty {
            odra_types::Type::Address => {
                let value = Address::deser(value).unwrap();
                Value::String(value.to_string())
            },
            odra_types::Type::Bool => {
                let value = bool::deser(value).unwrap();
                Value::Bool(value)
            },
            odra_types::Type::I32 => Value::Number(i32::deser(value).unwrap().into()),
            odra_types::Type::I64 => Value::Number(i64::deser(value).unwrap().into()),
            odra_types::Type::U8 => Value::Number(u8::deser(value).unwrap().into()),
            odra_types::Type::U32 => Value::Number(u32::deser(value).unwrap().into()),
            odra_types::Type::U64 => Value::Number(u64::deser(value).unwrap().into()),
            odra_types::Type::U128 => todo!(),
            odra_types::Type::U256 => Value::String(U256::deser(value).unwrap().into()),
            odra_types::Type::U512 => Value::String(U512::deser(value).unwrap().into()),
            odra_types::Type::Unit => Value::Null,
            odra_types::Type::String => Value::String(String::deser(value).unwrap()),
            odra_types::Type::Option(ty) => {
                if value == vec![110, 117, 108, 108] {
                    Value::Null
                } else {
                    Self::parse_value(key, value, *ty)
                }
            },
            odra_types::Type::Result { ok, err } => todo!(),
            odra_types::Type::Map { key, value } => todo!(),
            odra_types::Type::Tuple1(_) => todo!(),
            odra_types::Type::Tuple2(_) => todo!(),
            odra_types::Type::Tuple3(_) => todo!(),
            odra_types::Type::Any => todo!(),
            odra_types::Type::Vec(_) => todo!()
        }
    }

    /// Retrieves a vector of argument names.
    pub fn arg_names(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

#[cfg(test)]
mod test {
    use serde_json::{Value, Number};

    use crate::CallArgs;

    #[test]
    fn test_args() {
        let mut args = CallArgs::new();
        args.insert("val", 1);
        args.insert("opt", None::<i32>);
        // args.insert("vec", vec![1, 2, 3]);
        args.insert("str", String::from("txt"));

        // assert_eq!(args.get::<Option<u32>>("opt"), Some(1));
        // assert_eq!(args.get::<Vec<u32>>("vec"), vec![1, 2, 3]);
        // assert_eq!(args.get_as_value("opt"), Value::Number(Number::from(1)));
        assert_eq!(args.get_as_value("opt"), Value::Null);
    }
}
