//! Module containing DogContract. It is used in docs to explain how to interact with the storage.
use odra::prelude::*;
use odra::{Mapping, Var};

type FriendName = String;
type Visits = u32;

/// A simple contract that represents a second dog.
#[odra::module]
pub struct DogContract2 {
    name: Var<String>,
    friends: Mapping<FriendName, Visits>
}

#[odra::module]
impl DogContract2 {
    /// Initializes the contract with the given parameters.
    pub fn init(&mut self, name: String) {
        self.name.set(name);
    }

    /// Returns the dog's name.
    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    /// Adds a visit to the friend's visits.
    pub fn visit(&mut self, friend_name: &FriendName) {
        let visits = self.visits(friend_name);
        self.friends.set(friend_name, visits + 1);
    }

    /// Returns the total visits of the friend.
    pub fn visits(&self, friend_name: &FriendName) -> u32 {
        self.friends.get_or_default(friend_name)
    }
}

#[cfg(test)]
mod tests {
    use super::{DogContract2, DogContract2InitArgs};
    use odra::{host::Deployer, prelude::string::ToString};

    #[test]
    fn visit_test() {
        let test_env = odra_test::env();
        let mut dog_contract = DogContract2::deploy(
            &test_env,
            DogContract2InitArgs {
                name: "Mantus".to_string()
            }
        );
        assert_eq!(dog_contract.visits(&"Kuba".to_string()), 0);
        dog_contract.visit(&"Kuba".to_string());
        assert_eq!(dog_contract.visits(&"Kuba".to_string()), 1);
    }
}
