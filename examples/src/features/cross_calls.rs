use odra::prelude::*;
use odra::{module::Module, Address, UnwrapOrRevert, Variable};

#[odra::module]
pub struct CrossContract {
    pub math_engine: Variable<Address>
}

#[odra::module]
impl CrossContract {
    pub fn init(&mut self, math_engine_address: Address) {
        self.math_engine.set(math_engine_address);
    }

    pub fn add_using_another(&self) -> u32 {
        let math_engine_address = self.math_engine.get().unwrap_or_revert(&self.env());
        MathEngineContractRef::new(self.env(), math_engine_address).add(3, 5)
    }
}

#[odra::module]
pub struct MathEngine {}

#[odra::module]
impl MathEngine {
    pub fn add(&self, n1: u32, n2: u32) -> u32 {
        n1 + n2
    }
}

#[cfg(test)]
mod tests {
    use super::{CrossContractDeployer, MathEngineDeployer};

    #[test]
    fn test_cross_calls() {
        let test_env = odra_test::env();
        let math_engine_contract = MathEngineDeployer::init(&test_env);
        let cross_contract =
            CrossContractDeployer::init(&test_env, *math_engine_contract.address());

        assert_eq!(cross_contract.add_using_another(), 8);
    }
}
