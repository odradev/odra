use std::collections::BTreeMap;

use crate::MockVMType;

#[derive(Default, Debug)]
pub struct CallArgs {
    data: BTreeMap<String, Vec<u8>>
}

impl CallArgs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: MockVMType>(&mut self, key: &str, value: T) {
        // TODO: Handle unwrap.
        self.data.insert(String::from(key), value.ser().unwrap());
    }

    pub fn get<T: MockVMType>(&self, key: &str) -> T {
        // TODO: Handle unwraps.
        T::deser(self.data.get(key).unwrap().clone()).unwrap()
    }

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
