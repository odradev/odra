use crate::docs::cross_calls::MathEngine;

#[odra::module]
pub struct MyContract {
    pub math_engine: MathEngine,
}

#[odra::module]
impl MyContract {
    pub fn add_using_module(&self) -> u32 {
        self.math_engine.add(3, 5)
    }
}

#[cfg(test)]
mod tests {
    use super::MyContractDeployer;

    #[test]
    fn test_modules() {
        let my_contract = MyContractDeployer::default();
        assert_eq!(my_contract.add_using_module(), 8);
    }
}
