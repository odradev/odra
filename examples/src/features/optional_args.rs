//! Optional arguments example.

use odra::prelude::*;
use odra::{args::Maybe, Var};

/// Contract structure.
#[odra::module]
pub struct Token {
    name: Var<String>,
    metadata: Var<String>
}

#[odra::module]
impl Token {
    /// Initializes the contract with the given name and optional metadata.
    /// Maybe is different from Option in that the Option value is always present, but
    /// it can be either Some or None.
    pub fn init(&mut self, name: String, metadata: Maybe<String>) {
        self.name.set(name);
        self.metadata.set(metadata.unwrap_or_default());
    }

    /// Returns the token metadata.
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
