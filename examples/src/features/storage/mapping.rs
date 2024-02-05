use odra::prelude::*;
use odra::{Mapping, Variable};

type FriendName = String;
type Visits = u32;

#[odra::module]
pub struct DogContract2 {
    name: Variable<String>,
    friends: Mapping<FriendName, Visits>
}

#[odra::module]
impl DogContract2 {
    pub fn init(&mut self, name: String) {
        self.name.set(name);
    }

    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    pub fn visit(&mut self, friend_name: FriendName) {
        let visits = self.visits(friend_name.clone());
        self.friends.set(&friend_name, visits + 1);
    }

    pub fn visits(&self, friend_name: FriendName) -> u32 {
        self.friends.get_or_default(&friend_name)
    }
}

#[cfg(test)]
mod tests {
    use super::{DogContract2HostRef, DogContract2InitArgs};
    use odra::{host::Deployer, prelude::string::ToString};

    #[test]
    fn visit_test() {
        let test_env = odra_test::env();
        let mut dog_contract = DogContract2HostRef::deploy(
            &test_env,
            DogContract2InitArgs {
                name: "Mantus".to_string()
            }
        );
        assert_eq!(dog_contract.visits("Kuba".to_string()), 0);
        dog_contract.visit("Kuba".to_string());
        assert_eq!(dog_contract.visits("Kuba".to_string()), 1);
    }
}
