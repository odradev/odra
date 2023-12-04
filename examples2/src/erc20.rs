use odra::{casper_event_standard, Module, OdraError};
use odra::{prelude::*, CallDef, Event, ModuleWrapper};
use odra::{Address, ContractEnv, Mapping, Variable, U256, U512};

#[derive(Event, Eq, PartialEq, Debug)]
pub struct Transfer {
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub amount: U256
}

#[derive(Event, Eq, PartialEq, Debug)]
pub struct CrossTransfer {
    pub from: Option<Address>,
    pub to: Option<Address>,
    pub other_contract: Address,
    pub amount: U256
}

#[derive(Event, Eq, PartialEq, Debug)]
pub struct Approval {
    pub owner: Address,
    pub spender: Address,
    pub value: U256
}

#[repr(u16)]
pub enum Erc20Error {
    InsufficientBalance = 1,
    InsufficientAllowance = 2
}

impl From<Erc20Error> for OdraError {
    fn from(error: Erc20Error) -> Self {
        OdraError::user(error as u16)
    }
}

// #[odra::module(events = [Transfer, Approval]])]
pub struct Erc20 {
    env: Rc<ContractEnv>,
    total_supply: Variable<U256>,
    balances: Mapping<Address, U256>,
    // ownable: ModuleWrapper<Ownable>
}

// impl HasEvents for Erc20 {
//     fn events() -> Vec<Event> {
//         vec![Event::from_type::<Transfer>(), Event::from_type::<Approval>()]
//             .concat(Variable<U256>::events())
//             .concat(Mapping::<Address, U256>::events())
//             .concat(ModuleWrapper::<Ownable>::events())
//     }
// }

#[odra_macros::module]
impl Erc20 {
    pub fn init(&mut self, total_supply: Option<U256>) {
        if let Some(total_supply) = total_supply {
            self.total_supply.set(total_supply);
            self.balances.set(self.env().caller(), total_supply);
        }
    }

    pub fn approve(&mut self, to: Address, amount: U256) {
        self.env.emit_event(Approval {
            owner: self.env.caller(),
            spender: to,
            value: amount
        });
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get_or_default()
    }

    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get_or_default(owner)
    }

    pub fn transfer2(&mut self, to: Address, value: U256) {
        self.transfer(to, value);
    }

    #[odra::guard]
    pub fn transfer(&mut self, to: Address, value: U256) {
        let caller = self.env().caller();
        let balances = &mut self.balances;
        let from_balance = balances.get_or_default(caller);
        let to_balance = balances.get_or_default(to);
        if from_balance < value {
            self.env().revert(Erc20Error::InsufficientBalance)
        }
        balances.set(caller, from_balance.saturating_sub(value));
        balances.set(to, to_balance.saturating_add(value));
        self.env.emit_event(Transfer {
            from: Some(caller),
            to: Some(to),
            amount: value
        });
    }

    pub fn cross_total(&self, other: Address) -> U256 {
        let other_erc20 = Erc20ContractRef {
            address: other,
            env: self.env()
        };

        self.total_supply() + other_erc20.total_supply()
    }

    pub fn pay_to_mint(&mut self) {
        let attached_value = self.env().attached_value();
        let caller = self.env().caller();
        let caller_balance = self.balance_of(caller);
        self.balances
            .set(caller, caller_balance + U256::from(attached_value.as_u64()));
        self.total_supply
            .set(self.total_supply() + U256::from(attached_value.as_u64()));
    }

    pub fn get_current_block_time(&self) -> u64 {
        self.env().get_block_time()
    }

    pub fn burn_and_get_paid(&mut self, amount: U256) {
        let caller = self.env().caller();
        let caller_balance = self.balance_of(caller);
        if amount > caller_balance {
            self.env().revert(Erc20Error::InsufficientBalance)
        }

        self.balances.set(caller, caller_balance - amount);
        self.total_supply.set(self.total_supply() - amount);
        self.env()
            .transfer_tokens(&caller, &U512::from(amount.as_u64()));
    }

    pub fn cross_transfer(&mut self, other: Address, to: Address, value: U256) {
        let caller = self.env().caller();

        let mut other_erc20 = Erc20ContractRef {
            address: other,
            env: self.env()
        };

        other_erc20.transfer(to, value);
        self.env.emit_event(CrossTransfer {
            from: Some(self.env.self_address()),
            to: Some(to),
            other_contract: other,
            amount: value
        });
    }
}

// autogenerated for general purpose module.
mod __erc20_module {
    use super::Erc20;
    use odra::{module::Module, prelude::*, ContractEnv, Mapping, Variable};

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

#[cfg(odra_module = "Erc20")]
mod __erc20_schema {
    use odra::{contract_def::ContractBlueprint2, prelude::String};

    #[no_mangle]
    fn module_schema() -> ContractBlueprint2 {
        ContractBlueprint2 {
            name: String::from("Erc20"),
            // events: <Erc20 as HasEvents>::events(),
        }
    }
}

// autogenerated for the WasmContractEnv.
#[cfg(odra_module = "Erc20")]
#[cfg(target_arch = "wasm32")]
mod __erc20_wasm_parts {
    use super::{Approval, Erc20, Transfer};
    use odra::casper_event_standard::Schemas;
    use odra::casper_types::{
        CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Group,
        Parameter, RuntimeArgs
    };
    use odra::odra_casper_wasm_env;
    use odra::odra_casper_wasm_env::casper_contract::contract_api::runtime;
    use odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert;
    use odra::odra_casper_wasm_env::WasmContractEnv;
    use odra::Module;
    use odra::{prelude::*, ContractEnv};
    use odra::{runtime_args, Address, U256};

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
        entry_points.add_entry_point(EntryPoint::new(
            "pay_to_mint",
            alloc::vec![],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "get_current_block_time",
            alloc::vec![],
            u64::cl_type(),
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "burn_and_get_paid",
            alloc::vec![Parameter::new("amount", U256::cl_type()),],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "approve",
            alloc::vec![
                Parameter::new("to", Address::cl_type()),
                Parameter::new("amount", U256::cl_type()),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points.add_entry_point(EntryPoint::new(
            "cross_transfer",
            alloc::vec![
                Parameter::new("other", Address::cl_type()),
                Parameter::new("to", Address::cl_type()),
                Parameter::new("value", U256::cl_type()),
            ],
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract
        ));
        entry_points
    }

    pub fn execute_call() {
        let schemas: Schemas = __erc20_schema::events().into(); //.with::<Transfer>().with::<Approval>();
        let total_supply: Option<U256> = runtime::get_named_arg("total_supply");
        let init_args = runtime_args! {
            "total_supply" => total_supply
        };
        odra::odra_casper_wasm_env::host_functions::install_contract(
            entry_points(),
            schemas,
            Some(init_args)
        );
    }

    pub fn execute_init() {
        let total_supply: Option<U256> = runtime::get_named_arg("total_supply");
        let env = WasmContractEnv::new_env();
        let mut contract: Erc20 = Erc20::new(Rc::new(env));
        contract.init(total_supply);
    }

    pub fn execute_init(env: Rc<ContractEnv>) -> () {
        let total_supply: Option<U256> = env.get_named_arg("total_supply");
        let mut contract: Erc20 = Erc20::new(env);
        contract.init(total_supply)
    }

    pub fn execute_total_supply() {
        let env = WasmContractEnv::new_env();
        let contract: Erc20 = Erc20::new(Rc::new(env));
        let result = contract.total_supply();
        runtime::ret(CLValue::from_t(result).unwrap_or_revert())
    }

    pub fn execute_balance_of() {
        let owner: Address = runtime::get_named_arg("owner");
        let env = WasmContractEnv::new_env();
        let contract: Erc20 = Erc20::new(Rc::new(env));
        let result = contract.balance_of(owner);
        runtime::ret(CLValue::from_t(result).unwrap_or_revert())
    }

    pub fn execute_transfer() {
        let to: Address = runtime::get_named_arg("to");
        let value: U256 = runtime::get_named_arg("value");
        let env = WasmContractEnv::new_env();
        let mut contract: Erc20 = Erc20::new(Rc::new(env));
        contract.transfer(to, value);
    }

    pub fn execute_cross_total() {
        let other: Address = runtime::get_named_arg("other");
        let env = WasmContractEnv::new_env();
        let contract: Erc20 = Erc20::new(Rc::new(env));
        let result = contract.cross_total(other);
        runtime::ret(CLValue::from_t(result).unwrap_or_revert())
    }

    pub fn execute_pay_to_mint() {
        let env = WasmContractEnv::new_env();
        odra::odra_casper_wasm_env::host_functions::handle_attached_value();
        let mut contract: Erc20 = Erc20::new(Rc::new(env));
        contract.pay_to_mint();
        odra::odra_casper_wasm_env::host_functions::clear_attached_value();
    }

    pub fn execute_get_current_block_time() {
        let env = WasmContractEnv::new_env();
        let contract: Erc20 = Erc20::new(Rc::new(env));
        let result = contract.get_current_block_time();
        runtime::ret(CLValue::from_t(result).unwrap_or_revert())
    }

    pub fn execute_burn_and_get_paid() {
        let amount: U256 = runtime::get_named_arg("amount");
        let env = WasmContractEnv::new_env();
        let mut contract: Erc20 = Erc20::new(Rc::new(env));
        contract.burn_and_get_paid(amount);
    }

    pub fn execute_approve() {
        let to: Address = runtime::get_named_arg("to");
        let amount: U256 = runtime::get_named_arg("amount");
        let env = WasmContractEnv::new_env();
        let mut contract: Erc20 = Erc20::new(Rc::new(env));
        contract.approve(to, amount);
    }

    pub fn execute_cross_transfer() {
        let other: Address = runtime::get_named_arg("other");
        let to: Address = runtime::get_named_arg("to");
        let value: U256 = runtime::get_named_arg("value");
        let env = WasmContractEnv::new_env();
        let mut contract: Erc20 = Erc20::new(Rc::new(env));
        contract.cross_transfer(other, to, value);
    }

    #[no_mangle]
    fn call() {
        execute_call();
    }

    #[no_mangle]
    fn init() {
        let env = WasmContractEnv::new_env();
        let result = execute_init(Rc::new(env));
        // if returns
        runtime::ret(CLValue::from_t(result).unwrap_or_revert())
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

    #[no_mangle]
    fn pay_to_mint() {
        execute_pay_to_mint();
    }

    #[no_mangle]
    fn get_current_block_time() {
        execute_get_current_block_time();
    }

    #[no_mangle]
    fn burn_and_get_paid() {
        execute_burn_and_get_paid();
    }

    #[no_mangle]
    fn approve() {
        execute_approve();
    }

    #[no_mangle]
    fn cross_transfer() {
        execute_cross_transfer();
    }
}

use odra::{runtime_args, ExecutionError, RuntimeArgs};

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    pub use super::*;
    use odra::casper_types::system::mint::Error::InsufficientFunds;
    use odra::CallResult;
    use odra::ExecutionError;
    use odra::OdraError;
    use odra::U512;
    use odra::{Bytes, ToBytes};

    #[test]
    fn erc20_works() {
        let env = odra::test_env();
        let alice = env.get_account(0);
        let bob = env.get_account(1);

        // Deploy the contract as Alice.
        let mut erc20 = Erc20Deployer::init(&env, Some(100.into()));
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

        // Test cross calls
        let mut pobcoin = Erc20Deployer::init(&env, Some(100.into()));
        assert_eq!(erc20.cross_total(pobcoin.address.clone()), 200.into());

        // Test attaching value and balances
        let initial_balance = U512::from(100000000000000000u64);
        assert_eq!(env.balance_of(&erc20.address), 0.into());
        assert_eq!(env.balance_of(&alice), initial_balance);

        env.set_caller(alice);
        pobcoin.with_tokens(100.into()).pay_to_mint();
        assert_eq!(env.balance_of(&pobcoin.address), 100.into());
        assert_eq!(pobcoin.total_supply(), 200.into());
        assert_eq!(pobcoin.balance_of(alice), 100.into());
        assert_eq!(pobcoin.balance_of(bob), 100.into());

        assert_eq!(env.balance_of(&alice), initial_balance - U512::from(100));
        assert_eq!(env.balance_of(&pobcoin.address), 100.into());

        // Test block time
        let block_time = pobcoin.get_current_block_time();
        env.advance_block_time(12345);
        let new_block_time = pobcoin.get_current_block_time();
        assert_eq!(block_time + 12345, new_block_time);

        // Test transfer from contract to account
        env.set_caller(alice);
        let current_balance = env.balance_of(&alice);
        pobcoin.burn_and_get_paid(100.into());
        assert_eq!(env.balance_of(&alice), current_balance + U512::from(100));

        env.print_gas_report()
    }

    #[test]
    fn erc20_call_result() {
        let env = odra::test_env();
        let alice = env.get_account(0);
        let bob = env.get_account(1);

        // Deploy the contract as Alice.
        let mut erc20 = Erc20Deployer::init(&env, Some(100.into()));
        let mut pobcoin = Erc20Deployer::init(&env, Some(100.into()));

        // Make a call or two
        erc20.transfer(bob, 10.into());
        erc20.transfer(bob, 30.into());

        // Test call result
        assert_eq!(env.events_count(&erc20.address), 2);

        let call_result = env.last_call();
        assert!(call_result.result.is_ok());
        assert_eq!(call_result.contract_address, erc20.address);
        assert_eq!(call_result.caller, alice);
        assert_eq!(call_result.result(), vec![].into());
        assert_eq!(
            call_result.contract_events(&erc20.address),
            vec![Bytes::from(
                Transfer {
                    from: Some(alice),
                    to: Some(bob),
                    amount: 30.into()
                }
                .to_bytes()
                .unwrap()
            )]
        );

        // call with error
        erc20.try_transfer(bob, 100_000_000.into()).unwrap_err();
        let call_result = env.last_call();
        assert!(call_result.result.is_err());
        assert_eq!(call_result.contract_address, erc20.address);
        assert_eq!(call_result.caller, alice);
        assert_eq!(call_result.events.get(&erc20.address).unwrap(), &vec![]);

        // cross call
        pobcoin.transfer(erc20.address, 100.into());
        erc20.cross_transfer(pobcoin.address, alice, 50.into());
        let call_result = env.last_call();

        assert_eq!(
            call_result.contract_events(&pobcoin.address),
            vec![Bytes::from(
                Transfer {
                    from: Some(erc20.address),
                    to: Some(alice),
                    amount: 50.into()
                }
                .to_bytes()
                .unwrap()
            )]
        );
        assert_eq!(
            call_result.contract_events(&erc20.address),
            vec![Bytes::from(
                CrossTransfer {
                    from: Some(erc20.address),
                    to: Some(alice),
                    other_contract: pobcoin.address,
                    amount: 50.into()
                }
                .to_bytes()
                .unwrap()
            )]
        );
    }

    #[test]
    fn erc20_events_work() {
        let env = odra::test_env();
        let alice = env.get_account(0);
        let bob = env.get_account(1);
        let charlie = env.get_account(2);

        // Deploy the contract as Alice.
        let mut erc20 = Erc20Deployer::init(&env, Some(100.into()));

        // Emit some events
        erc20.transfer(bob, 10.into());
        erc20.approve(bob, 10.into());
        erc20.transfer(charlie, 20.into());

        // Test events
        let event: Transfer = env.get_event(&erc20.address, 0).unwrap();
        assert_eq!(
            event,
            Transfer {
                from: Some(alice),
                to: Some(bob),
                amount: 10.into()
            }
        );

        let event: Approval = env.get_event(&erc20.address, 1).unwrap();
        assert_eq!(
            event,
            Approval {
                owner: alice,
                spender: bob,
                value: 10.into()
            }
        );

        let event: Transfer = env.get_event(&erc20.address, 2).unwrap();
        assert_eq!(
            event,
            Transfer {
                from: Some(alice),
                to: Some(charlie),
                amount: 20.into()
            }
        );

        // Test negative indices
        let event: Transfer = env.get_event(&erc20.address, -1).unwrap();
        assert_eq!(
            event,
            Transfer {
                from: Some(alice),
                to: Some(charlie),
                amount: 20.into()
            }
        );
    }

    #[test]
    fn erc20_events_testing_work() {
        // Given
        let env = odra::test_env();
        let alice = env.get_account(0);
        let bob = env.get_account(1);

        // Deploy the contract as Alice.
        let mut erc20 = Erc20Deployer::init(&env, Some(100.into()));

        // When event is emitted
        erc20.approve(bob, 10.into());
        erc20.transfer(bob, 10.into());
        let first_emitted = Approval {
            owner: alice,
            spender: bob,
            value: 10.into()
        };
        let emitted_in_second_call = Transfer {
            from: Some(alice),
            to: Some(bob),
            amount: 10.into()
        };
        let not_emitted = CrossTransfer {
            from: Some(alice),
            to: Some(bob),
            other_contract: bob,
            amount: 10.into()
        };

        // Then we can check it
        // If contract emitted a specific event during whole lifetime
        assert!(env.emitted(&erc20.address, "Transfer"));
        // or all of them
        assert_eq!(
            env.event_names(&erc20.address),
            vec!["Approval".to_string(), "Transfer".to_string()]
        );

        // We can limit our checks to a last call
        assert_eq!(
            erc20.last_call().event_names(),
            vec!["Transfer".to_string()]
        );
        // or
        erc20.last_call().emitted("Transfer");

        // We can check the whole event, not only names:
        // TODO: change it to hopefully_emitted.into()
        assert_eq!(
            erc20.last_call().events(),
            vec![Bytes::from(emitted_in_second_call.to_bytes().unwrap())]
        );
        // or
        assert!(erc20.last_call().emitted_event(&emitted_in_second_call));
        // or for whole lifetime
        assert!(env.emitted_event(&erc20.address, &emitted_in_second_call));

        // To check the order of events, use the power of vec:
        assert_eq!(
            env.events(&erc20.address)[0..2],
            [
                Bytes::from(first_emitted.to_bytes().unwrap()),
                Bytes::from(emitted_in_second_call.to_bytes().unwrap())
            ]
            .to_vec()
        );

        assert!(env
            .event_names(&erc20.address)
            .ends_with(vec!["Approval".to_string(), "Transfer".to_string()].as_slice()));

        // Counter examples
        assert!(!erc20.last_call().emitted("Approval"));
        assert!(!env.emitted(&erc20.address, "CrossTransfer"));
        assert!(!env.emitted_event(&erc20.address, &not_emitted));
    }

    #[test]
    fn test_erc20_errors() {
        let env = odra::test_env();
        let alice = env.get_account(0);

        // Deploy the contract as Alice.
        let mut erc20 = Erc20Deployer::init(&env, Some(100.into()));

        // Test errors
        let result = erc20.try_transfer(alice, 1_000_000.into());
        assert_eq!(result, Err(Erc20Error::InsufficientBalance.into()));

        // With return value
        let result = erc20.try_balance_of(alice);
        assert_eq!(result, Ok(100.into()));
    }

    #[test]
    fn erc20_xxx() {
        let env = odra::test_env();
        let alice = env.get_account(0);
        let bob = env.get_account(1);

        // Deploy the contract as Alice.
        let mut erc20 = Erc20Deployer::init(&env, Some(100.into()));
        assert_eq!(erc20.total_supply(), 100.into());
        assert_eq!(erc20.balance_of(alice), 100.into());
        assert_eq!(erc20.balance_of(bob), 0.into());

        // Transfer 10 tokens from Alice to Bob.
        erc20.transfer(bob, 10.into());
        assert_eq!(erc20.balance_of(alice), 90.into());
        assert_eq!(erc20.balance_of(bob), 10.into());

        env.print_gas_report();
    }

}
