use crate::features::cross_calls::MathEngine;
use odra::prelude::*;
use odra::ModuleWrapper;

#[odra::module]
pub struct ModulesContract {
    pub math_engine: ModuleWrapper<MathEngine>
}

#[odra::module]
impl ModulesContract {
    pub fn add_using_module(&self) -> u32 {
        self.math_engine.add(3, 5)
    }
}

#[cfg(test)]
mod tests {
    use super::ModulesContractDeployer;

    #[test]
    fn test_modules() {
        let test_env = odra_test::test_env();
        let modules_contract = ModulesContractDeployer::init(&test_env);
        assert_eq!(modules_contract.add_using_module(), 8);
    }
}
