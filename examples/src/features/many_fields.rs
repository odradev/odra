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

/// Simulates the original contract before upgrade (15 fields).
#[allow(dead_code)]
#[odra::module]
pub struct OriginalContract {
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
    f15: Var<u32>
}

#[odra::module]
impl OriginalContract {
    pub fn set_all(&mut self) {
        self.f1.set(100);
        self.f10.set(1000);
        self.f15.set(1500);
    }

    pub fn get_f1(&self) -> u32 {
        self.f1.get_or_default()
    }

    pub fn get_f10(&self) -> u32 {
        self.f10.get_or_default()
    }

    pub fn get_f15(&self) -> u32 {
        self.f15.get_or_default()
    }
}

/// Simulates the upgraded contract (same first 15 fields + 5 new ones).
#[allow(dead_code)]
#[odra::module]
pub struct UpgradedContract {
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
impl UpgradedContract {
    pub fn get_f1(&self) -> u32 {
        self.f1.get_or_default()
    }

    pub fn get_f10(&self) -> u32 {
        self.f10.get_or_default()
    }

    pub fn get_f15(&self) -> u32 {
        self.f15.get_or_default()
    }

    pub fn get_f16(&self) -> u32 {
        self.f16.get_or_default()
    }

    pub fn set_f16(&mut self, v: u32) {
        self.f16.set(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, InstallConfig, NoArgs};
    use odra::prelude::Addressable;

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

    #[test]
    fn upgrade_preserves_existing_fields() {
        let env = odra_test::env();

        // Deploy "original" contract and write data
        let mut original = OriginalContract::deploy_with_cfg(
            &env,
            NoArgs,
            InstallConfig::upgradable::<OriginalContract>()
        );
        original.set_all();
        assert_eq!(original.get_f1(), 100);
        assert_eq!(original.get_f10(), 1000);
        assert_eq!(original.get_f15(), 1500);

        // "Upgrade": replace the contract implementation at the same address
        let addr = original.address();
        let mut upgraded = UpgradedContract::try_upgrade(&env, addr, NoArgs).unwrap();

        // Original fields must still be readable with same values
        assert_eq!(upgraded.get_f1(), 100);
        assert_eq!(upgraded.get_f10(), 1000);
        assert_eq!(upgraded.get_f15(), 1500);

        // New fields start empty
        assert_eq!(upgraded.get_f16(), 0);

        // Can write to new fields without affecting old ones
        upgraded.set_f16(1600);
        assert_eq!(upgraded.get_f16(), 1600);
        assert_eq!(upgraded.get_f1(), 100);
    }
}
