use crate::features::cross_calls::MathEngine;
use odra::prelude::*;
use odra::SubModule;

/// Contract that uses a module.
#[odra::module]
pub struct ModulesContract {
    /// Math engine module.
    pub math_engine: SubModule<MathEngine>
}

#[odra::module]
impl ModulesContract {
    /// Adds 3 and 5 using the math engine module.
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
