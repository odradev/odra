#![allow(missing_docs)]
use odra::{prelude::*, Var};

type IPv4 = [u8; 4];
type IPv6 = [u8; 16];

/// An enum representing an IP address.
#[odra::odra_type]
#[derive(Default)]
pub enum IP {
    /// No data.
    #[default]
    Unknown,
    /// Single unnamed element.
    IPv4(IPv4),
    /// multiple unnamed elements.
    IPv4WithDescription(IPv4, String),
    /// single named element.
    IPv6 { ip: IPv6 },
    /// multiple named elements.
    IPv6WithDescription { ip: IPv6, description: String }
}

#[odra::odra_type]
pub enum Fieldless {
    /// Tuple variant.
    Tuple(),
    /// Struct variant.
    Struct {},
    /// Unit variant.
    Unit
}

/// Unit-only enum.
#[odra::odra_type]
#[derive(Default)]
pub enum Unit {
    #[default]
    A = 10,
    B = 20,
    C
}

/// A struct with named elements.
#[odra::odra_type]
pub struct MyStruct {
    a: u32,
    b: u32
}

// A struct with unnamed elements cannot be an Odra type.
// #[odra::odra_type]
// pub struct TupleStruct(u32, u32);

#[odra::module]
pub struct MyContract {
    ip: Var<IP>,
    fieldless: Var<Fieldless>,
    unit: Var<Unit>,
    my_struct: Var<MyStruct>
}

#[odra::odra_error]
pub enum Errors {
    NotFound = 1
}

#[odra::module]
impl MyContract {
    pub fn init(&mut self, ip: IP, fieldless: Fieldless, unit: Unit, my_struct: MyStruct) {
        self.ip.set(ip);
        self.fieldless.set(fieldless);
        self.unit.set(unit);
        self.my_struct.set(my_struct);
    }

    pub fn get_ip(&self) -> IP {
        self.ip.get_or_default()
    }

    pub fn get_fieldless(&self) -> Fieldless {
        self.fieldless.get_or_revert_with(Errors::NotFound)
    }

    pub fn get_unit(&self) -> Unit {
        self.unit.get_or_default()
    }

    pub fn get_struct(&self) -> MyStruct {
        self.my_struct.get_or_revert_with(Errors::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::Deployer;

    #[test]
    fn test_contract() {
        let test_env = odra_test::env();
        let init_args = MyContractInitArgs {
            ip: IP::IPv4([192, 168, 0, 1]),
            fieldless: Fieldless::Tuple(),
            unit: Unit::C,
            my_struct: MyStruct { a: 10, b: 20 }
        };
        let contract = MyContract::deploy(&test_env, init_args);

        assert_eq!(contract.get_ip(), IP::IPv4([192, 168, 0, 1]));
        assert_eq!(contract.get_fieldless(), Fieldless::Tuple());
        assert_eq!(contract.get_unit(), Unit::C);
        assert_eq!(contract.get_struct(), MyStruct { a: 10, b: 20 });
    }
}
