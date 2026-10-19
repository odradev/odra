//! Private functions of a module impl are not entry points: they may use the names reserved for
//! entry points and arguments. The generated code calls `<Module as odra::module::Module>::new`
//! and friends fully qualified, so an inherent helper of the same name does not clash.
#![deny(warnings)]

use odra::prelude::*;

#[odra::module]
pub struct Module0 {
    value: Var<u64>
}

#[odra::module]
impl Module0 {
    pub fn init(&mut self) {
        let value = self.new(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.new(3, 4)
    }

    fn new(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module]
pub struct Module1 {
    value: Var<u64>
}

#[odra::module]
impl Module1 {
    pub fn init(&mut self) {
        let value = self.env(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.env(3, 4)
    }

    fn env(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module]
pub struct Module2 {
    value: Var<u64>
}

#[odra::module]
impl Module2 {
    pub fn init(&mut self) {
        let value = self.address(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.address(3, 4)
    }

    fn address(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module]
pub struct Module3 {
    value: Var<u64>
}

#[odra::module]
impl Module3 {
    pub fn init(&mut self) {
        let value = self.new_contract(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.new_contract(3, 4)
    }

    fn new_contract(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module]
pub struct Module4 {
    value: Var<u64>
}

#[odra::module]
impl Module4 {
    pub fn init(&mut self) {
        let value = self.upgrade_child_contract(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.upgrade_child_contract(3, 4)
    }

    fn upgrade_child_contract(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module]
pub struct Module5 {
    value: Var<u64>
}

#[odra::module]
impl Module5 {
    pub fn init(&mut self) {
        let value = self.batch_upgrade_child_contract(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.batch_upgrade_child_contract(3, 4)
    }

    fn batch_upgrade_child_contract(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module(factory = on)]
pub struct Factory0 {
    value: Var<u64>
}

#[odra::module(factory = on)]
impl Factory0 {
    pub fn init(&mut self) {
        let value = self.new(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.new(3, 4)
    }

    fn new(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module(factory = on)]
pub struct Factory1 {
    value: Var<u64>
}

#[odra::module(factory = on)]
impl Factory1 {
    pub fn init(&mut self) {
        let value = self.env(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.env(3, 4)
    }

    fn env(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module(factory = on)]
pub struct Factory2 {
    value: Var<u64>
}

#[odra::module(factory = on)]
impl Factory2 {
    pub fn init(&mut self) {
        let value = self.address(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.address(3, 4)
    }

    fn address(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module(factory = on)]
pub struct Factory3 {
    value: Var<u64>
}

#[odra::module(factory = on)]
impl Factory3 {
    pub fn init(&mut self) {
        let value = self.new_contract(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.new_contract(3, 4)
    }

    fn new_contract(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module(factory = on)]
pub struct Factory4 {
    value: Var<u64>
}

#[odra::module(factory = on)]
impl Factory4 {
    pub fn init(&mut self) {
        let value = self.upgrade_child_contract(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.upgrade_child_contract(3, 4)
    }

    fn upgrade_child_contract(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

#[odra::module(factory = on)]
pub struct Factory5 {
    value: Var<u64>
}

#[odra::module(factory = on)]
impl Factory5 {
    pub fn init(&mut self) {
        let value = self.batch_upgrade_child_contract(1, 2);
        self.value.set(value);
    }

    #[odra(payable)]
    pub fn get(&self) -> u64 {
        self.batch_upgrade_child_contract(3, 4)
    }

    fn batch_upgrade_child_contract(&self, gas: u64, attached_value: u64) -> u64 {
        gas + attached_value + self.value.get_or_default()
    }
}

fn main() {}
