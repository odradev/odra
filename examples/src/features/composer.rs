use odra::Variable;

#[odra::module]
pub struct SharedStorage {
    pub value: Variable<String>
}

#[odra::module]
pub struct MyStorage {
    pub shared: SharedStorage,
    pub version: Variable<u8>
}

#[odra::module]
pub struct ComposableContract {
    pub shared: SharedStorage,
    #[odra(using = "shared")]
    pub storage: MyStorage
}

#[odra::module]
impl ComposableContract {
    #[odra(init)]
    pub fn init(&mut self, version: u8, value: String) {
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

#[cfg(test)]
mod test {
    use crate::features::composer::ComposableContractDeployer;

    #[test]
    fn t() {
        let shared_value = "shared_value".to_string();
        let token = ComposableContractDeployer::init(1, shared_value.clone());

        assert_eq!(token.get_value(), shared_value);

        assert_eq!(token.get_value_via_storage(), shared_value);
    }
}
