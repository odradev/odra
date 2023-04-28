use odra::{Instance, Variable};

#[odra::module]
pub struct SharedStorage {
    pub value: Variable<String>
}

#[odra::module]
pub struct MyStorage {
    pub shared: SharedStorage,
    pub version: Variable<u8>
}

#[odra::module(skip_instance)]
pub struct ComposableContract {
    pub shared: SharedStorage,
    pub storage: MyStorage
}

#[odra::module]
impl ComposableContract {
    #[odra(init)]
    pub fn init(&mut self, version: &u8, value: &String) {
        self.storage.version.set(version);
        self.shared.value.set(value);
    }

    pub fn get_value(&self) -> String {
        self.shared.value.get_or_default()
    }

    pub fn get_value_via_storage(&self) -> String {
        self.storage.shared.value.get_or_default()
    }
}

impl Instance for ComposableContract {
    fn instance(namespace: &str) -> Self {
        let shared = SharedStorageComposer::new(namespace, "shared").compose();
        let storage = MyStorageComposer::new(namespace, "storage")
            .with_shared(&shared)
            .compose();
        Self { shared, storage }
    }
}

#[cfg(test)]
mod test {
    use crate::composer::ComposableContractDeployer;

    #[test]
    fn t() {
        let shared_value = "shared_value".to_string();
        let token = ComposableContractDeployer::init(&1, &shared_value);

        assert_eq!(token.get_value(), shared_value);

        assert_eq!(token.get_value_via_storage(), shared_value);
    }
}
