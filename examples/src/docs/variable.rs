use odra::Variable;

#[odra::module]
pub struct DogContract {
    barks: Variable<bool>,
    weight: Variable<u32>,
    name: Variable<String>,
    walks: Variable<Vec<u32>>
}

#[odra::module]
impl DogContract {
    #[odra(init)]
    pub fn init(&mut self, barks: bool, weight: u32, name: String) {
        self.barks.set(&barks);
        self.weight.set(&weight);
        self.name.set(&name);
        self.walks.set(&Vec::<u32>::default());
    }

    pub fn barks(&self) -> bool {
        self.barks.get_or_default()
    }

    pub fn weight(&self) -> u32 {
        self.weight.get_or_default()
    }

    pub fn name(&self) -> String {
        self.name.get_or_default()
    }

    pub fn walks_amount(&self) -> u32 {
        let walks = self.walks.get_or_default();
        walks.len() as u32
    }

    pub fn walks_total_length(&self) -> u32 {
        let walks = self.walks.get_or_default();
        walks.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::DogContractDeployer;

    #[test]
    fn init_test() {
        let dog_contract = DogContractDeployer::init(true, 10, "Mantus".to_string());
        assert!(dog_contract.barks());
        assert_eq!(dog_contract.weight(), 10);
        assert_eq!(dog_contract.name(), "Mantus".to_string());
        assert_eq!(dog_contract.walks_amount(), 0);
        assert_eq!(dog_contract.walks_total_length(), 0);
    }
}
