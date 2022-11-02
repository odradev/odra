use std::collections::BTreeMap;

use crate::MockVMType;

#[derive(Default, Debug)]
pub struct CallArgs {
    data: BTreeMap<String, Vec<u8>>,
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

// TODO: Tests.
