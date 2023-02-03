use std::collections::BTreeMap;

use odra_types::Type;
use serde_json::Value;

use crate::Typed;

/// Represents a collection of arguments passed to a smart contract entrypoint call.
#[derive(Default, Debug)]
pub struct CallArgs {
    data: BTreeMap<String, (String, Type)>
}

impl CallArgs {
    /// Creates a new no-args instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a new empty arg into the collection.
    pub fn insert<K: Into<String>, V: ToString + Typed>(&mut self, key: K, value: V) {
        // TODO: Handle unwrap.
        self.data.insert(key.into(), (value.to_string(), V::ty()));
    }

    /// Gets an argument by the name.
    pub fn get(&self, key: &str) -> (String, Type) {
        // TODO: Handle unwraps.
        self.data.get(key).cloned().unwrap()
    }

    pub fn get_as_value(&self, key: &str) -> Value {
        let (value, ty) = self.get(key);
        match ty {
            odra_types::Type::Address => Value::String(value),
            odra_types::Type::Bool => Value::Bool(value.parse().unwrap()),
            odra_types::Type::I32 => Value::Number(value.parse().unwrap()),
            odra_types::Type::I64 => Value::Number(value.parse().unwrap()),
            odra_types::Type::U8 => Value::Number(value.parse().unwrap()),
            odra_types::Type::U32 => Value::Number(value.parse().unwrap()),
            odra_types::Type::U64 => Value::Number(value.parse().unwrap()),
            odra_types::Type::U128 => Value::Number(value.parse().unwrap()),
            odra_types::Type::U256 => Value::Number(value.parse().unwrap()),
            odra_types::Type::U512 => Value::Number(value.parse().unwrap()),
            odra_types::Type::Unit => Value::Null,
            odra_types::Type::String => Value::String(value),
            odra_types::Type::Option(_) => todo!(),
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

// #[cfg(test)]
// mod test {
//     use crate::CallArgs;

//     #[test]
//     fn test_args() {
//         let mut args = CallArgs::new();
//         args.insert("val", 1);
//         args.insert("opt", Some(1));
//         args.insert("vec", vec![1, 2, 3]);
//         args.insert("str", String::from("txt"));

//         assert_eq!(args.get::<u32>("val"), 1);
//         // assert_eq!(args.get::<Option<u32>>("opt"), Some(1));
//         // assert_eq!(args.get::<Vec<u32>>("vec"), vec![1, 2, 3]);
//         assert_eq!(args.get::<String>("str"), String::from("txt"));
//     }
// }
