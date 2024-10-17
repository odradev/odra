//! Module containing DogContract. It is used in docs to explain how to interact with the storage.
use odra::prelude::*;

/// A simple contract that represents a dog.
#[odra::module]
pub struct DogContract {
    barks: Var<bool>,
    weight: Var<u32>,
    name: Var<String>,
    walks: Var<Vec<u32>>
}

#[odra::module]
impl DogContract {
    /// Initializes the contract with the given parameters.
    pub fn init(&mut self, barks: bool, weight: u32, name: String) {
        self.barks.set(barks);
        self.weight.set(weight);
        self.name.set(name);
        self.walks.set(Vec::<u32>::default());
    }

    /// Returns true if the dog barks.
    pub fn barks(&self) -> bool {
        self.barks.get_or_default()
    }

    /// Returns the dog's weight.
    pub fn weight(&self) -> u32 {
        self.weight.get_or_default()
    }

    /// Returns the dog's name.
    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    /// Adds a walk to the dog's walks.
    pub fn walks_amount(&self) -> u32 {
        let walks = self.walks.get_or_default();
        walks.len() as u32
    }

    /// Returns the total length of the dog's walks.
    pub fn walks_total_length(&self) -> u32 {
        let walks = self.walks.get_or_default();
        walks.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::{DogContract, DogContractInitArgs};
    use odra::{host::Deployer, prelude::string::ToString};

    #[test]
    fn init_test() {
        let test_env = odra_test::env();
        let init_args = DogContractInitArgs {
            barks: true,
            weight: 10,
            name: "Mantus".to_string()
        };
        let dog_contract = DogContract::deploy(&test_env, init_args);
        assert!(dog_contract.barks());
        assert_eq!(dog_contract.weight(), 10);
        assert_eq!(dog_contract.name(), "Mantus".to_string());
        assert_eq!(dog_contract.walks_amount(), 0);
        assert_eq!(dog_contract.walks_total_length(), 0);
    }
}
