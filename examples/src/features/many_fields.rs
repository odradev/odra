//! Tests for contracts with many storage fields (>15) to verify storage key encoding.
use odra::prelude::*;

/// A contract with 20 fields to test storage key encoding beyond the 15-field threshold.
#[allow(dead_code)]
#[odra::module]
pub struct ManyFieldsContract {
    f1: Var<u32>,
    f2: Var<u32>,
    f3: Var<u32>,
    f4: Var<u32>,
    f5: Var<u32>,
    f6: Var<u32>,
    f7: Var<u32>,
    f8: Var<u32>,
    f9: Var<u32>,
    f10: Var<u32>,
    f11: Var<u32>,
    f12: Var<u32>,
    f13: Var<u32>,
    f14: Var<u32>,
    f15: Var<u32>,
    f16: Var<u32>,
    f17: Var<u32>,
    f18: Var<u32>,
    f19: Var<u32>,
    f20: Var<u32>
}

#[odra::module]
impl ManyFieldsContract {
    pub fn set_values(&mut self) {
        self.f1.set(1);
        self.f15.set(15);
        self.f16.set(16);
        self.f20.set(20);
    }

    pub fn get_f1(&self) -> u32 {
        self.f1.get_or_default()
    }

    pub fn get_f15(&self) -> u32 {
        self.f15.get_or_default()
    }

    pub fn get_f16(&self) -> u32 {
        self.f16.get_or_default()
    }

    pub fn get_f20(&self) -> u32 {
        self.f20.get_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn many_fields_storage_works() {
        let env = odra_test::env();
        let mut contract = ManyFieldsContract::deploy(&env, NoArgs);
        contract.set_values();
        assert_eq!(contract.get_f1(), 1);
        assert_eq!(contract.get_f15(), 15);
        assert_eq!(contract.get_f16(), 16);
        assert_eq!(contract.get_f20(), 20);
    }
}
