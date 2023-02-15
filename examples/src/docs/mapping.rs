use odra::{Mapping, Variable};

#[odra::module]
pub struct DogContract2 {
    name: Variable<String>,
    friends: Mapping<String, u32>,
}

#[odra::module]
impl DogContract2 {
    #[odra(init)]
    pub fn init(&mut self, name: String) {
        self.name.set(name);
    }

    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    pub fn visit(&mut self, friend_name: String) {
        let visits = self.visits(friend_name.clone());
        self.friends.set(&friend_name, visits + 1);
    }

    pub fn visits(&self, friend_name: String) -> u32 {
        match self.friends.get(&friend_name) {
            None => {
                0
            },
            Some(v) => {
                v
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DogContract2Deployer;

    #[test]
    fn visit_test() {
        let mut dog_contract = DogContract2Deployer::init("Mantus".to_string());
        assert_eq!(dog_contract.visits("Kuba".to_string()), 0);
        dog_contract.visit("Kuba".to_string());
        assert_eq!(dog_contract.visits("Kuba".to_string()), 1);
    }
}
