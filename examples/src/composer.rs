use odra::{types::{Address, U256}, Mapping, Variable, Instance};

#[derive(Clone)]
#[odra::module]
pub struct Refs {
    pub values: Mapping<String, Address>,
    pub metadata: RefsMetadata
}

#[derive(Clone)]
#[odra::module]
pub struct RefsMetadata {
    pub version: Variable<String>
}

#[odra::module]
pub struct TokenStorage {
    pub refs: Refs,
    pub total_supply: Variable<U256>
}

#[odra::module(skip_instance)]
pub struct Token {
    pub refs: Refs,
    pub storage: TokenStorage
}

#[odra::module]
impl Token {
    #[odra(init)]
    pub fn init(&mut self) {
        self.storage.total_supply.set(U256::from(100));
    }

    pub fn namespace(&self) -> String {
        self.storage.total_supply.path().to_string()
    }
}

impl Instance for Token {
    fn instance(namespace: &str) -> Self {
        let refs = RefsComposer::new(&format!("refs_{}", namespace))
            .with_metadata(RefsMetadataComposer::new(namespace)
                .with_version(Instance::instance("ver"))
                .compose())
            .compose();
        let storage = TokenStorageComposer::new(&format!("storage_{}", namespace))
            // .with_refs(refs.clone())
            .compose();
        Self {
            refs,
            storage
        }
    }
}

#[cfg(test)]
mod test {
    use crate::composer::TokenDeployer;

    #[test]
    fn t() {
        let token = TokenDeployer::init();
        dbg!(token.namespace());

        // let a = &[stringify!("s"), "ss"]
        //     .iter()
        //     .filter(|str| str.is_empty())
        //     .collect::<Vec<_>>()
        //     .()
        //     .join("_");
        assert!(false);
    }
}