//! Module containing DogContract. It is used in docs to explain how to interact with the storage.
use odra::{prelude::*, List, Var};

/// A simple contract that represents a third dog.
#[odra::module]
pub struct DogContract3 {
    name: Var<String>,
    walks: List<u32>
}

#[odra::module]
impl DogContract3 {
    /// Initializes the contract with the given parameters.
    pub fn init(&mut self, name: String) {
        self.name.set(name);
    }

    /// Returns the dog's name.
    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    /// Returns the amount of walks the dog has taken.
    pub fn walks_amount(&self) -> u32 {
        self.walks.len()
    }

    /// Returns the total length of the dog's walks.
    pub fn walks_total_length(&self) -> u32 {
        self.walks.iter().sum()
    }

    /// Adds a walk to the dog's walks.
    pub fn walk_the_dog(&mut self, length: u32) {
        self.walks.push(length);
    }
}

#[cfg(test)]
mod tests {
    use super::{DogContract3HostRef, DogContract3InitArgs};
    use odra::{host::Deployer, prelude::string::ToString};

    #[test]
    fn init_test() {
        let test_env = odra_test::env();
        let mut dog_contract = DogContract3HostRef::deploy(
            &test_env,
            DogContract3InitArgs {
                name: "DogContract".to_string()
            }
        );
        assert_eq!(dog_contract.walks_amount(), 0);
        assert_eq!(dog_contract.walks_total_length(), 0);
        dog_contract.walk_the_dog(5);
        dog_contract.walk_the_dog(10);
        assert_eq!(dog_contract.walks_amount(), 2);
        assert_eq!(dog_contract.walks_total_length(), 15);
    }
}
