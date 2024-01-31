use odra::prelude::*;
use odra::{module::Module, Variable};

#[odra::module]
pub struct ReentrancyMock {
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
            ReentrancyMockContractRef::new(self.env(), self.env().self_address())
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
    use super::ReentrancyMockDeployer;

    #[test]
    fn non_reentrant_function_can_be_called() {
        let mut contract = ReentrancyMockDeployer::init(&odra_test::env());
        assert_eq!(contract.get_value(), 0);
        contract.non_reentrant_count();
        assert_eq!(contract.get_value(), 1);
    }

    #[test]
    fn ref_recursion_not_allowed() {
        let mut contract = ReentrancyMockDeployer::init(&odra_test::env());
        assert_eq!(
            contract.try_count_ref_recursive(11).unwrap_err(),
            odra::ExecutionError::ReentrantCall.into()
        );
    }

    #[test]
    fn local_recursion_allowed() {
        let mut contract = ReentrancyMockDeployer::init(&odra_test::env());
        contract.count_local_recursive(11);
        assert_eq!(contract.get_value(), 11);
    }
}
