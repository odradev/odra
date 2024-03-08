#![allow(missing_docs)]

use odra::{args::Maybe, Var};
use odra::prelude::*;

#[odra::module]
pub struct Token {
    name: Var<String>,
    metadata: Var<String>,
}

#[odra::module]
impl Token {
    pub fn init(&mut self, name: String, metadata: Maybe<String>) {
        self.name.set(name.clone());
        self.metadata.set(metadata.unwrap_or_default());
    }

    pub fn metadata(&self) -> String {
        self.metadata.get_or_default()
    }
}


#[cfg(test)]
mod test {
    use crate::features::optional_args::TokenInitArgs;

    use super::*;
    use odra::host::Deployer;

    #[test]
    fn test_no_opt_arg() {
        let test_env = odra_test::env();
        let init_args = TokenInitArgs { 
            name: String::from("MyToken"), 
            metadata: Maybe::None
        };
        let my_contract = TokenHostRef::deploy(&test_env, init_args);
        assert_eq!(my_contract.metadata(), String::from(""));
    }

    #[test]
    fn test_with_opt_arg() {
        let test_env = odra_test::env();
        let init_args = TokenInitArgs { 
            name: String::from("MyToken"), 
            metadata: Maybe::Some(String::from("MyMetadata"))
        };
        let my_contract = TokenHostRef::deploy(&test_env, init_args);
        assert_eq!(my_contract.metadata(), String::from("MyMetadata"));
    }
}
