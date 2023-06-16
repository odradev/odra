use crate::features::cross_calls::MathEngine;
use alloc::{vec::Vec, string::String};

#[odra::module]
pub struct ModulesContract {
    pub math_engine: MathEngine
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
        let modules_contract = ModulesContractDeployer::default();
        assert_eq!(modules_contract.add_using_module(), 8);
    }
}
