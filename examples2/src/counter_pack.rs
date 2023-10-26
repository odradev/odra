use crate::counter::Counter;
use odra2::prelude::*;
use odra2::ContractEnv;
use odra2::Mapping;
use odra2::ModuleWrapper;

pub struct CounterPack {
    env: Rc<ContractEnv>,
    counter0: ModuleWrapper<Counter>,
    counter1: ModuleWrapper<Counter>,
    counter2: ModuleWrapper<Counter>,
    counter3: ModuleWrapper<Counter>,
    counter4: ModuleWrapper<Counter>,
    counter5: ModuleWrapper<Counter>,
    counter6: ModuleWrapper<Counter>,
    counter7: ModuleWrapper<Counter>,
    counter8: ModuleWrapper<Counter>,
    counter9: ModuleWrapper<Counter>,
    counters: Mapping<(u8, u8), u32>,
    counters_map: Mapping<u8, Counter>
}

impl CounterPack {
    pub fn get_count(&self, index_a: u8, index_b: u8) -> u32 {
        match index_a {
            0 => self.counter0.get_count(index_b),
            1 => self.counter1.get_count(index_b),
            2 => self.counter2.get_count(index_b),
            3 => self.counter3.get_count(index_b),
            4 => self.counter4.get_count(index_b),
            5 => self.counter5.get_count(index_b),
            6 => self.counter6.get_count(index_b),
            7 => self.counter7.get_count(index_b),
            8 => self.counter8.get_count(index_b),
            9 => self.counter9.get_count(index_b),
            _ => unreachable!()
        }
        // self.counters.get_or_default((index_a, index_b))
        // self.counters_map.module(index_a).get_count(index_b)
    }

    pub fn increment(&mut self, index_a: u8, index_b: u8) {
        match index_a {
            0 => self.counter0.increment(index_b),
            1 => self.counter1.increment(index_b),
            2 => self.counter2.increment(index_b),
            3 => self.counter3.increment(index_b),
            4 => self.counter4.increment(index_b),
            5 => self.counter5.increment(index_b),
            6 => self.counter6.increment(index_b),
            7 => self.counter7.increment(index_b),
            8 => self.counter8.increment(index_b),
            9 => self.counter9.increment(index_b),
            _ => unreachable!()
        };
        // let count = self.counters.get_or_default((index_a, index_b));
        // self.counters.set((index_a, index_b), count + 1);
        // self.counters_map.module(index_a).increment(index_b);
    }
}

// autogenerated
mod odra_core_module {
    use super::*;

    impl Module for CounterPack {
        fn new(env: Rc<ContractEnv>) -> Self {
            let counter0 = ModuleWrapper::new(Rc::clone(&env), 0);
            let counter1 = ModuleWrapper::new(Rc::clone(&env), 1);
            let counter2 = ModuleWrapper::new(Rc::clone(&env), 2);
            let counter3 = ModuleWrapper::new(Rc::clone(&env), 3);
            let counter4 = ModuleWrapper::new(Rc::clone(&env), 4);
            let counter5 = ModuleWrapper::new(Rc::clone(&env), 5);
            let counter6 = ModuleWrapper::new(Rc::clone(&env), 6);
            let counter7 = ModuleWrapper::new(Rc::clone(&env), 7);
            let counter8 = ModuleWrapper::new(Rc::clone(&env), 8);
            let counter9 = ModuleWrapper::new(Rc::clone(&env), 9);
            let counters = Mapping::new(Rc::clone(&env), 10);
            let counters_map = Mapping::new(Rc::clone(&env), 11);
            Self {
                env,
                counter0,
                counter1,
                counter2,
                counter3,
                counter4,
                counter5,
                counter6,
                counter7,
                counter8,
                counter9,
                counters,
                counters_map
            }
        }

        fn env(&self) -> Rc<ContractEnv> {
            self.env.clone()
        }
    }
}

#[cfg(odra_module = "CounterPack")]
#[cfg(target_arch = "wasm32")]
mod __counter_pack_wasm_parts {
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

    use super::CounterPack;

    extern crate alloc;

    pub fn entry_points() -> EntryPoints {
        let mut entry_points = EntryPoints::new();
        entry_points.add_entry_point(EntryPoint::new(
            "get_count",
            alloc::vec![
                Parameter::new("index_a", CLType::U8),
                Parameter::new("index_b", CLType::U8),
            ],
            CLType::U32,
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "increment",
            alloc::vec![
                Parameter::new("index_a", CLType::U8),
                Parameter::new("index_b", CLType::U8),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points
    }

    pub fn execute_call() {
        odra_casper_backend2::wasm_host::install_contract(entry_points(), Schemas::new(), None);
    }

    pub fn execute_get_count() {
        let index_a: u8 = runtime::get_named_arg("index_a");
        let index_b: u8 = runtime::get_named_arg("index_b");
        let env = WasmContractEnv::new();
        let contract: CounterPack = CounterPack::new(Rc::new(env));
        let result = contract.get_count(index_a, index_b);
        runtime::ret(CLValue::from_t(result).unwrap_or_revert());
    }

    pub fn execute_increment() {
        let index_a: u8 = runtime::get_named_arg("index_a");
        let index_b: u8 = runtime::get_named_arg("index_b");
        let env = WasmContractEnv::new();
        let mut contract: CounterPack = CounterPack::new(Rc::new(env));
        contract.increment(index_a, index_b);
    }

    #[no_mangle]
    fn call() {
        execute_call();
    }

    #[no_mangle]
    fn get_count() {
        execute_get_count();
    }

    #[no_mangle]
    fn increment() {
        execute_increment();
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod __counter_pack_test_parts {
    use odra2::types::{runtime_args, Bytes, RuntimeArgs, ToBytes, U256, U512};
    use odra2::{prelude::*, EntryPointsCaller};
    use odra2::{types::Address, CallDef, HostEnv};

    use crate::counter_pack::CounterPack;

    pub struct CounterPackHostRef {
        address: Address,
        env: HostEnv
    }

    impl CounterPackHostRef {
        pub fn get_count(&self, index_a: u8, index_b: u8) -> u32 {
            self.env.call_contract(
                &self.address,
                CallDef::new(
                    String::from("get_count"),
                    runtime_args! {
                        "index_a" => index_a,
                        "index_b" => index_b
                    }
                )
            )
        }

        pub fn increment(&self, index_a: u8, index_b: u8) {
            self.env.call_contract(
                &self.address,
                CallDef::new(
                    String::from("increment"),
                    runtime_args! {
                        "index_a" => index_a,
                        "index_b" => index_b
                    },
                )
            )
        }
    }

    pub struct CounterPackDeployer;

    impl CounterPackDeployer {
        pub fn init(env: &HostEnv) -> CounterPackHostRef {
            let epc = EntryPointsCaller::new(env.clone(), |contract_env, call_def| {
                use odra2::types::ToBytes;
                let mut counter_pack = CounterPack::new(Rc::new(contract_env));
                match call_def.method() {
                    "get_count" => {
                        let index_a: u8 = call_def.get("index_a").unwrap();
                        let index_b: u8 = call_def.get("index_b").unwrap();
                        let result = counter_pack.get_count(index_a, index_b);
                        Bytes::from(result.to_bytes().unwrap())
                    }
                    "increment" => {
                        let index_a: u8 = call_def.get("index_a").unwrap();
                        let index_b: u8 = call_def.get("index_b").unwrap();
                        let result = counter_pack.increment(index_a, index_b);
                        Bytes::from(result.to_bytes().unwrap())
                    }
                    _ => panic!("Unknown method")
                }
            });

            let address = env.new_contract("counter_pack", None, Some(epc));

            CounterPackHostRef {
                address,
                env: env.clone()
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use __counter_pack_test_parts::*;

#[cfg(test)]
mod tests {
    pub use super::*;

    #[test]
    fn counter_pack_works() {
        let env = odra2::test_env();
        let counter_pack = CounterPackDeployer::init(&env);

        let n: u8 = 3;
        let m: u8 = 3;
        for i in 0..n {
            for j in 0..m {
                assert_eq!(counter_pack.get_count(i, j), 0);
                counter_pack.increment(i, j);
                assert_eq!(counter_pack.get_count(i, j), 1);
            }
        }

        env.print_gas_report();
    }
}
