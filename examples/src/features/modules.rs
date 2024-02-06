use crate::features::cross_calls::MathEngine;
use odra::prelude::*;
use odra::SubModule;

#[odra::module]
pub struct ModulesContract {
    pub math_engine: SubModule<MathEngine>
}

#[odra::module]
impl ModulesContract {
    pub fn add_using_module(&self) -> u32 {
        self.math_engine.add(3, 5)
    }
}

#[cfg(test)]
mod tests {
    use super::ModulesContractHostRef;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn test_modules() {
        let test_env = odra_test::env();
        let modules_contract = ModulesContractHostRef::deploy(&test_env, NoArgs);
        assert_eq!(modules_contract.add_using_module(), 8);
    }
}
