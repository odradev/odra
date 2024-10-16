use odra::prelude::*;

/// A module definition. Each module struct consists Vars and Mappings
/// or/and another modules.
#[odra::module]
pub struct Flapper {
    /// The module itself does not store the value,
    /// it's a proxy that writes/reads value to/from the host.
    value: Var<bool>,
}

/// Module implementation.
/// 
/// To generate entrypoints,
/// an implementation block must be marked as #[odra::module].
#[odra::module]
impl Flapper {
    /// Odra constructor.
    /// 
    /// Initializes the contract.
    pub fn init(&mut self) {
        self.value.set(false);
    }

    /// Replaces the current value with the passed argument.
    pub fn set(&mut self, value: bool) {
        self.value.set(value);
    }

    /// Replaces the current value with the opposite value.
    pub fn flap(&mut self) {
        self.value.set(!self.get());
    }

    /// Retrieves value from the storage. 
    /// If the value has never been set, the default value is returned.
    pub fn get(&self) -> bool {
        self.value.get_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::flapper::Flapper;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn flapping() {
        let env = odra_test::env();
        // To test a module we need to deploy it. `Flapper` implements `Deployer` trait, 
        // so we can use it to deploy the module.
        let mut contract = Flapper::deploy(&env, NoArgs);
        assert!(!contract.get());
        contract.flap();
        assert!(contract.get());
    }
}
