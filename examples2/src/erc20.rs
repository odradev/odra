use odra2::casper_event_standard::{self, Event};
use odra2::{prelude::*, ModuleWrapper, CallDef};
use odra2::{
    types::{Address, U256},
    ContractEnv, Mapping, Variable
};

#[derive(Event, Eq, PartialEq, Debug)]
pub struct Transfer {
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub amount: U256
}

#[derive(Event, Eq, PartialEq, Debug)]
pub struct Approval {
    pub owner: Address,
    pub spender: Address,
    pub value: U256
}

pub struct Erc20 {
    env: Rc<ContractEnv>,
    total_supply: Variable<U256>,
    balances: Mapping<Address, U256>
}

impl Erc20 {
    pub fn init(&mut self, total_supply: Option<U256>) {
        if let Some(total_supply) = total_supply {
            self.total_supply.set(total_supply);
            self.balances.set(self.env().caller(), total_supply);
        }
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get_or_default()
    }

    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get_or_default(owner)
    }

    pub fn transfer(&mut self, to: Address, value: U256) {
        let caller = self.env().caller();
        let balances = &mut self.balances;
        let from_balance = balances.get_or_default(caller);
        let to_balance = balances.get_or_default(to);
        if from_balance < value {
            self.env().revert(1);
        }
        balances.set(caller, from_balance - value);
        balances.set(to, to_balance + value);
    }

    pub fn cross_total(&self, other: Address) -> U256 {
        let other_erc20 = Erc20ContractRef {
            address: other,
            env: self.env()
        };

        self.total_supply() + other_erc20.total_supply()
    }
}

// autogenerated for general purpose module.
mod __erc20_module {
    use super::Erc20;
    use odra2::{module::Module, prelude::*, ContractEnv, Mapping, Variable};

    impl Module for Erc20 {
        fn new(env: Rc<ContractEnv>) -> Self {
            let total_supply = Variable::new(Rc::clone(&env), 1);
            let balances = Mapping::new(Rc::clone(&env), 2);
            Self {
                env,
                total_supply,
                balances
            }
        }

        fn env(&self) -> Rc<ContractEnv> {
            self.env.clone()
        }
    }
}

pub struct Erc20ContractRef {
        pub address: Address,
        pub env: Rc<ContractEnv>
    }

impl Erc20ContractRef {
    pub fn total_supply(&self) -> U256 {
        self.env.call_contract(
            self.address,
            CallDef::new(String::from("total_supply"), RuntimeArgs::new(), None)
        )
    }
}



// autogenerated for the WasmContractEnv.
#[cfg(odra_module = "Erc20")]
#[cfg(target_arch = "wasm32")]
mod __erc20_wasm_parts {
    use super::{Approval, Erc20, Transfer};
    use odra2::casper_event_standard::Schemas;
    use odra2::odra_casper_backend2;
    use odra2::odra_casper_backend2::casper_contract::unwrap_or_revert::UnwrapOrRevert;
    use odra2::odra_casper_backend2::WasmContractEnv;
    use odra2::types::casper_types::{
        CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Group,
        Parameter, RuntimeArgs
    };
    use odra2::types::{runtime_args, Address, U256};
    use odra2::{prelude::*, ContractEnv};
    use odra_casper_backend2::casper_contract::contract_api::runtime;

    extern crate alloc;

    fn entry_points() -> EntryPoints {
        let mut entry_points = EntryPoints::new();
        entry_points.add_entry_point(EntryPoint::new(
            "init",
            alloc::vec![Parameter::new("total_supply", Option::<U256>::cl_type()),],
            CLType::Unit,
            EntryPointAccess::Groups(alloc::vec![Group::new("constructor_group")]),
            EntryPointType::Contract
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "total_supply",
            alloc::vec![],
            U256::cl_type(),
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "balance_of",
            alloc::vec![Parameter::new("owner", Address::cl_type()),],
            U256::cl_type(),
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "transfer",
            alloc::vec![
                Parameter::new("to", Address::cl_type()),
                Parameter::new("value", U256::cl_type()),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "cross_total",
            alloc::vec![Parameter::new("other", Address::cl_type()),],
            U256::cl_type(),
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points
    }

    pub fn execute_call() {
        let schemas = Schemas::new().with::<Transfer>().with::<Approval>();
        let total_supply: Option<U256> = runtime::get_named_arg("total_supply");
        let init_args = runtime_args! {
            "total_supply" => total_supply
        };
        odra_casper_backend2::wasm_host::install_contract(entry_points(), schemas, Some(init_args));
    }

    pub fn execute_init() {
        let total_supply: Option<U256> = runtime::get_named_arg("total_supply");
        let env = WasmContractEnv::new();
        let mut contract: Erc20 = Erc20::new(Rc::new(env));
        contract.init(total_supply);
    }

    pub fn execute_total_supply() {
        let env = WasmContractEnv::new();
        let contract: Erc20 = Erc20::new(Rc::new(env));
        let result = contract.total_supply();
        runtime::ret(CLValue::from_t(result).unwrap_or_revert())
    }

    pub fn execute_balance_of() {
        let owner: Address = runtime::get_named_arg("owner");
        let env = WasmContractEnv::new();
        let contract: Erc20 = Erc20::new(Rc::new(env));
        let result = contract.balance_of(owner);
        runtime::ret(CLValue::from_t(result).unwrap_or_revert())
    }

    pub fn execute_transfer() {
        let to: Address = runtime::get_named_arg("to");
        let value: U256 = runtime::get_named_arg("value");
        let env = WasmContractEnv::new();
        let mut contract: Erc20 = Erc20::new(Rc::new(env));
        contract.transfer(to, value);
    }

    pub fn execute_cross_total() {
        let other: Address = runtime::get_named_arg("other");
        let env = WasmContractEnv::new();
        let contract: Erc20 = Erc20::new(Rc::new(env));
        let result = contract.cross_total(other);
        runtime::ret(CLValue::from_t(result).unwrap_or_revert())
    }

    #[no_mangle]
    fn call() {
        execute_call();
    }

    #[no_mangle]
    fn init() {
        execute_init();
    }

    #[no_mangle]
    fn total_supply() {
        execute_total_supply();
    }

    #[no_mangle]
    fn balance_of() {
        execute_balance_of();
    }

    #[no_mangle]
    fn transfer() {
        execute_transfer();
    }

    #[no_mangle]
    fn cross_total() {
        execute_cross_total()
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod __erc20_test_parts {
    use crate::erc20::Erc20;
    use odra2::prelude::*;
    use odra2::types::casper_types::EntryPoints;
    use odra2::types::{runtime_args, Address, Bytes, RuntimeArgs, ToBytes, U256};
    use odra2::{CallDef, ContractEnv, EntryPointsCaller, HostEnv};

    pub struct Erc20HostRef {
        pub address: Address,
        pub env: HostEnv
    }

    impl Erc20HostRef {
        pub fn total_supply(&self) -> U256 {
            self.env.call_contract(
                &self.address,
                CallDef::new(String::from("total_supply"), RuntimeArgs::new(), None)
            )
        }

        pub fn balance_of(&self, owner: Address) -> U256 {
            self.env.call_contract(
                &self.address,
                CallDef::new(
                    String::from("balance_of"),
                    runtime_args! {
                        "owner" => owner
                    },
                    None
                )
            )
        }

        pub fn transfer(&self, to: Address, value: U256) {
            self.env.call_contract(
                &self.address,
                CallDef::new(
                    String::from("transfer"),
                    runtime_args! {
                        "to" => to,
                        "value" => value
                    },
                    None
                )
            )
        }

        pub fn cross_total(&self, other: Address) -> U256 {
            self.env.call_contract(
                &self.address,
                CallDef::new(
                    String::from("cross_total"),
                    runtime_args! {
                        "other" => other
                    },
                    None
                )
            )
        }
    }

    pub struct Erc20Deployer;

    impl Erc20Deployer {
        pub fn init(env: &HostEnv, total_supply: Option<U256>) -> Erc20HostRef {
            let epc = EntryPointsCaller::new(env.clone(), |contract_env, call_def| {
                use odra2::types::ToBytes;
                let mut erc20 = Erc20::new(Rc::new(contract_env));
                match call_def.method() {
                    "init" => {
                        let total_supply: Option<U256> = call_def.get("total_supply").unwrap();
                        let result = erc20.init(total_supply);
                        Bytes::from(result.to_bytes().unwrap())
                    }
                    "total_supply" => {
                        let result = erc20.total_supply();
                        Bytes::from(result.to_bytes().unwrap())
                    }
                    "balance_of" => {
                        let owner: Address = call_def.get("owner").unwrap();
                        let result = erc20.balance_of(owner);
                        Bytes::from(result.to_bytes().unwrap())
                    }
                    "transfer" => {
                        let to: Address = call_def.get("to").unwrap();
                        let value: U256 = call_def.get("value").unwrap();
                        let result = erc20.transfer(to, value);
                        Bytes::from(result.to_bytes().unwrap())
                    }
                    "cross_total" => {
                        let other: Address = call_def.get("other").unwrap();
                        let result = erc20.cross_total(other);
                        Bytes::from(result.to_bytes().unwrap())
                    }
                    _ => panic!("Unknown method")
                }
            });

            let address = env.new_contract(
                "erc20",
                Some(runtime_args! {
                    "total_supply" => total_supply
                }),
                Some(epc)
            );

            Erc20HostRef {
                address,
                env: env.clone()
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use __erc20_test_parts::*;
use odra2::types::RuntimeArgs;

#[cfg(test)]
mod tests {
    use odra2::types::U512;
    pub use super::*;
    use odra2::types::ToBytes;

    #[test]
    fn erc20_works() {
        let env = odra2::test_env();
        let alice = env.get_account(0);
        let bob = env.get_account(1);

        // Deploy the contract as Alice.
        let erc20 = Erc20Deployer::init(&env, Some(100.into()));
        assert_eq!(erc20.total_supply(), 100.into());
        assert_eq!(erc20.balance_of(alice), 100.into());
        assert_eq!(erc20.balance_of(bob), 0.into());

        // Transfer 10 tokens from Alice to Bob.
        erc20.transfer(bob, 10.into());
        assert_eq!(erc20.balance_of(alice), 90.into());
        assert_eq!(erc20.balance_of(bob), 10.into());

        // Transfer 10 tokens back to Alice.
        env.set_caller(bob);
        erc20.transfer(alice, 10.into());
        assert_eq!(erc20.balance_of(alice), 100.into());
        assert_eq!(erc20.balance_of(bob), 0.into());

        // // Test cross calls
        let pobcoin = Erc20Deployer::init(&env, Some(100.into()));
        assert_eq!(erc20.cross_total(pobcoin.address.clone()), 200.into());

        env.print_gas_report();
    }
}
