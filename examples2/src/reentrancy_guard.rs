use odra::{prelude::*, ContractEnv, Variable};

#[odra::module]
pub struct ReentrancyMock {
    env: Rc<ContractEnv>,
    counter: Variable<u32>
}

#[odra::module]
impl ReentrancyMock {
    #[odra(non_reentrant)]
    pub fn count_local_recursive(&mut self, n: u32) {
        if n > 0 {
            self.count();
            self.count_local_recursive(n - 1);
        }
    }

    #[odra(non_reentrant)]
    pub fn count_ref_recursive(&mut self, n: u32) {
        if n > 0 {
            self.count();
            let other_erc20 = ReentrancyMockContractRef {
                address: self.env.self_address(),
                env: self.env.clone()
            }
            .count_ref_recursive(n - 1);
        }
    }

    #[odra(non_reentrant)]
    pub fn non_reentrant_count(&mut self) {
        self.count();
    }

    pub fn get_value(&self) -> u32 {
        self.counter.get_or_default()
    }
}

impl ReentrancyMock {
    fn count(&mut self) {
        let c = self.counter.get_or_default();
        self.counter.set(c + 1);
    }
}

#[cfg(test)]
mod test {
    use odra::test_env;

    use super::ReentrancyMockDeployer;

    #[test]
    fn non_reentrant_function_can_be_called() {
        let env = odra::test_env();

        let mut contract = ReentrancyMockDeployer::init(&env);
        assert_eq!(contract.get_value(), 0);
        contract.non_reentrant_count();
        assert_eq!(contract.get_value(), 1);
    }

    #[test]
    fn ref_recursion_not_allowed() {
        let env = odra::test_env();
        let mut contract = ReentrancyMockDeployer::init(&env);
        assert!(contract.try_count_ref_recursive(11).is_err());
    }

    #[test]
    fn local_recursion_allowed() {
        let env = odra::test_env();
        let mut contract = ReentrancyMockDeployer::init(&env);
        contract.count_local_recursive(11);
        assert_eq!(contract.get_value(), 11);
    }
}
