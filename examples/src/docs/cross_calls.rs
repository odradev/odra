use odra::Variable;
use odra::types::{Address};

#[odra::module]
pub struct MyContract {
    pub math_engine: Variable<Address>,
}

#[odra::module]
impl MyContract {
    #[odra(init)]
    pub fn init(&mut self, math_engine_address: Address) {
        self.math_engine.set(math_engine_address);
    }

    pub fn add_using_another(&self) -> u32 {
        let math_engine_address = self.math_engine.get().unwrap();
        MathEngineRef::at(math_engine_address).add(3, 5)
    }
}

#[odra::module]
pub struct MathEngine {
}

#[odra::module]
impl MathEngine {
    pub fn add(&self, n1: u32, n2: u32) -> u32 {
        n1 + n2
    }
}

#[cfg(test)]
mod tests {
    use super::{MyContractDeployer, MathEngineDeployer};

    #[test]
    fn test_cross_calls() {
        let math_engine_contract = MathEngineDeployer::default();
        let my_contract = MyContractDeployer::init(math_engine_contract.address());

        assert_eq!(my_contract.add_using_another(), 8);
    }
}
