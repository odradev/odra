use std::collections::BTreeMap;

use crate::MockVMType;

/// Represents a collection of arguments passed to a smart contract entrypoint call.
#[derive(Default, Debug)]
pub struct CallArgs {
    data: BTreeMap<String, Vec<u8>>
}

impl CallArgs {
    /// Creates a new no-args instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a new empty arg into the collection.
    pub fn insert<K: Into<String>, T: MockVMType>(&mut self, key: K, value: T) {
        // TODO: Handle unwrap.
        self.data.insert(key.into(), value.ser().unwrap());
    }

    /// Gets an argument by the name.
    pub fn get<T: MockVMType>(&self, key: &str) -> T {
        // TODO: Handle unwraps.
        T::deser(self.data.get(key).unwrap().clone()).unwrap()
    }

    /// Retrieves a vector of argument names.
    pub fn arg_names(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

#[cfg(test)]
mod test {
    use crate::CallArgs;

    #[test]
    fn test_args() {
        let mut args = CallArgs::new();
        args.insert("val", 1);
        args.insert("opt", Some(1));
        args.insert("vec", vec![1, 2, 3]);
        args.insert("str", String::from("txt"));

        assert_eq!(args.get::<u32>("val"), 1);
        assert_eq!(args.get::<Option<u32>>("opt"), Some(1));
        assert_eq!(args.get::<Vec<u32>>("vec"), vec![1, 2, 3]);
        assert_eq!(args.get::<String>("str"), String::from("txt"));
    }
}
