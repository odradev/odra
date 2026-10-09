//! Livenet implementation of HostContext for HostEnv.

use crate::error;
use crate::livenet_contract_env::{call_locally, CallFrame, LivenetContractEnv};
use crate::panic_hook;
use odra_casper_rpc_client::casper_client::configuration::CasperClientConfiguration;
use odra_casper_rpc_client::casper_client::CasperClient;
use odra_casper_rpc_client::error::LivenetError;
use odra_casper_rpc_client::log::info;
use odra_casper_rpc_client::utils::find_wasm_file_path;
use odra_core::callstack::{Callstack, CallstackElement};
use odra_core::casper_types::Timestamp;
use odra_core::entry_point_callback::EntryPointsCaller;
use odra_core::{
    casper_types::{bytesrepr::Bytes, PublicKey, RuntimeArgs, U512},
    host::{HostContext, HostEnv, ThreadEnvFactory},
    CallDef, ContractEnv, GasReport
};
use odra_core::{prelude::*, EventError, VmError};
use odra_core::{ContractContainer, ContractRegister};
use std::fs;
use std::sync::RwLock;
use std::thread::sleep;

/// LivenetHost struct.
pub struct LivenetHost {
    casper_client: Rc<RefCell<CasperClient>>,
    contract_register: Rc<RwLock<ContractRegister>>,
    contract_env: Rc<ContractEnv>,
    callstack: Rc<RefCell<Callstack>>,
    /// The error of the last revert of a locally executed call.
    error: Rc<RefCell<Option<OdraError>>>
}

impl LivenetHost {
    /// Creates a new instance of LivenetHost.
    pub fn new() -> Rc<Self> {
        Rc::new(Self::new_instance().unwrap())
    }

    pub fn new_safe() -> Result<Rc<Self>, LivenetError> {
        let instance = Self::new_instance()?;
        Ok(Rc::new(instance))
    }

    fn new_instance() -> Result<Self, LivenetError> {
        let configuration = CasperClientConfiguration::from_env()?;
        Ok(Self::with_configuration(configuration))
    }

    fn with_configuration(configuration: CasperClientConfiguration) -> Self {
        let casper_client: Rc<RefCell<CasperClient>> =
            Rc::new(RefCell::new(CasperClient::new(configuration)));
        let callstack: Rc<RefCell<Callstack>> = Default::default();
        let contract_register = Rc::new(RwLock::new(Default::default()));
        let error: Rc<RefCell<Option<OdraError>>> = Default::default();
        let livenet_contract_env = LivenetContractEnv::new(
            casper_client.clone(),
            callstack.clone(),
            contract_register.clone(),
            error.clone()
        );
        let contract_env = Rc::new(ContractEnv::new(livenet_contract_env.clone()));
        livenet_contract_env
            .borrow()
            .set_contract_env(&contract_env);
        Self {
            casper_client,
            contract_register,
            contract_env,
            callstack,
            error
        }
    }

    /// Runs a non-mutable entry point on this machine, called by the caller of this host like a
    /// transaction would. A revert unwinds the call with a panic, it is caught here and returned
    /// as the error the contract reverted with.
    fn execute_locally(&self, address: &Address, call_def: CallDef) -> OdraResult<Bytes> {
        panic_hook::set_livenet_panic_hook();
        *self.error.borrow_mut() = None;
        let caller = CallstackElement::new_account(self.caller());
        let _caller_frame = CallFrame::push(&self.callstack, caller);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            call_locally(
                &self.callstack,
                &self.contract_register,
                (*self.contract_env).clone(),
                address,
                call_def
            )
        }));
        match result {
            Ok(result) => result,
            Err(_) => {
                let error = self.error.borrow_mut().take();
                Err(error.unwrap_or(OdraError::VmError(VmError::Panic)))
            }
        }
    }
}

impl HostContext for LivenetHost {
    fn set_caller(&self, caller: Address) {
        self.casper_client.borrow_mut().set_caller(caller);
    }

    fn set_gas(&self, gas: u64) {
        self.casper_client.borrow_mut().set_gas(gas);
    }

    fn caller(&self) -> Address {
        self.casper_client.borrow().caller()
    }

    fn get_account(&self, index: usize) -> Address {
        self.casper_client.borrow().get_account(index)
    }

    fn get_validator(&self, index: usize) -> PublicKey {
        let client = self.casper_client.borrow_mut();
        client.get_validator(index)
    }

    fn remove_validator(&self, _index: usize) {
        panic!("remove_validator is not supported on livenet");
    }

    fn get_storage_value(&self, address: &Address, key: &[u8]) -> Option<Bytes> {
        self.casper_client
            .borrow()
            .get_value(address, key)
            .unwrap_or_else(|e| read_failed("state value", address, e))
    }

    fn get_named_value(&self, address: &Address, name: &str) -> Option<Bytes> {
        self.casper_client
            .borrow()
            .get_named_value(address, name)
            .unwrap_or_else(|e| read_failed(name, address, e))
    }

    fn get_dictionary_value(
        &self,
        address: &Address,
        dictionary_name: &str,
        key: &[u8]
    ) -> Option<Bytes> {
        self.casper_client
            .borrow()
            .get_dictionary_value(address, dictionary_name, key)
            .unwrap_or_else(|e| read_failed(dictionary_name, address, e))
    }

    fn balance_of(&self, address: &Address) -> U512 {
        let client = self.casper_client.borrow();
        client.get_balance(address).unwrap_or_else(|e| {
            panic!(
                "Failed to get balance for address {:?}: {}",
                address,
                e.error_message()
            )
        })
    }

    fn advance_block_time(&self, time_diff: u64) {
        info(format!(
            "advance_block_time called - Waiting for {} ms",
            time_diff
        ));
        sleep(std::time::Duration::from_millis(time_diff));
    }

    fn advance_with_auctions(&self, diff: u64) {
        println!("advance_with_auctions called - Waiting for {diff} ms");
        sleep(std::time::Duration::from_millis(diff));
    }

    fn auction_delay(&self) -> u64 {
        let client = self.casper_client.borrow_mut();
        client.auction_delay()
    }

    fn unbonding_delay(&self) -> u64 {
        let client = self.casper_client.borrow_mut();
        client.unbonding_delay()
    }

    fn delegated_amount(&self, delegator: Address, validator: PublicKey) -> U512 {
        let client = self.casper_client.borrow_mut();
        client.delegated_amount(delegator, validator)
    }

    fn block_time(&self) -> u64 {
        let client = self.casper_client.borrow();
        client.get_block_time().unwrap()
    }

    fn thread_env_factory(&self) -> Option<ThreadEnvFactory> {
        // Every worker gets its own client (connection, caches) with this host's caller and gas.
        let client = self.casper_client.borrow();
        let caller = client.caller();
        let gas = client.gas().as_u64();
        Some(ThreadEnvFactory::new(move || {
            let env = HostEnv::new(LivenetHost::new());
            env.set_caller(caller);
            env.set_gas(gas);
            env
        }))
    }

    fn take_snapshot(&self) {
        panic!("Snapshots are not available on livenet: the state lives on a real chain")
    }

    fn restore_snapshot(&self) {
        panic!("Snapshots are not available on livenet: the state lives on a real chain")
    }

    fn get_event(&self, contract_address: &Address, index: u32) -> Result<Bytes, EventError> {
        let client = self.casper_client.borrow();
        client
            .get_event(contract_address, index)
            .map_err(|_| EventError::CouldntExtractEventData)
    }

    fn get_native_event(
        &self,
        contract_address: &Address,
        index: u32
    ) -> Result<Bytes, EventError> {
        // Only the native events of the transactions this environment sent, see
        // `CasperClient::native_events_count`.
        self.casper_client
            .borrow()
            .get_native_event(contract_address, index)
            .unwrap_or_else(|e| read_failed("native event", contract_address, e))
            .ok_or(EventError::IndexOutOfBounds)
    }

    fn get_events_count(&self, contract_address: &Address) -> Result<u32, EventError> {
        let client = self.casper_client.borrow();
        client
            .events_count(contract_address)
            .unwrap_or_else(|e| read_failed("events count", contract_address, e))
            .ok_or(EventError::CouldntExtractEventData)
    }

    fn get_native_events_count(&self, contract_address: &Address) -> Result<u32, EventError> {
        self.casper_client
            .borrow()
            .native_events_count(contract_address)
            .map_err(|e| read_failed("native events count", contract_address, e))
    }

    fn call_contract(
        &self,
        address: &Address,
        call_def: CallDef,
        use_proxy: bool
    ) -> OdraResult<Bytes> {
        if !call_def.is_mut() {
            return self.execute_locally(address, call_def);
        }
        let timestamp = Timestamp::now();
        let client = self.casper_client.borrow_mut();
        let result = match use_proxy {
            true => client.deploy_entrypoint_call_with_proxy(*address, call_def, timestamp),
            false => client.deploy_entrypoint_call(*address, call_def, timestamp)
        };
        result.map_err(|e| self.error_msg_to_odra_error(e.error_message(), Some(address)))
    }

    fn new_contract(
        &self,
        name: &str,
        init_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        let timestamp = Timestamp::now();
        let wasm_path = find_wasm_file_path(name)?;
        let wasm_bytes = fs::read(wasm_path).unwrap();
        let address = {
            let client = self.casper_client.borrow();
            match client.deploy_wasm(name, init_args, timestamp, wasm_bytes) {
                Ok(addr) => addr,
                Err(e) => {
                    log::error!("Error deploying contract: {}", e);
                    return Err(deployment_error(e, name));
                }
            }
        };
        self.register_contract(address, name.to_string(), entry_points_caller);
        Ok(address)
    }

    fn upgrade_contract(
        &self,
        name: &str,
        contract_to_upgrade: Address,
        upgrade_args: RuntimeArgs,
        entry_points_caller: EntryPointsCaller
    ) -> OdraResult<Address> {
        let timestamp = Timestamp::now();
        let wasm_path = find_wasm_file_path(name)?;
        let wasm_bytes = fs::read(wasm_path).unwrap();
        let client = self.casper_client.borrow();
        match client.deploy_wasm(name, upgrade_args, timestamp, wasm_bytes) {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error deploying contract: {}", e);
                return Err(deployment_error(e, name));
            }
        }
        self.register_contract(contract_to_upgrade, name.to_string(), entry_points_caller);
        Ok(contract_to_upgrade)
    }

    fn register_contract(
        &self,
        address: Address,
        contract_name: String,
        entry_points_caller: EntryPointsCaller
    ) {
        self.contract_register
            .write()
            .expect("Couldn't write contract register.")
            .add(
                address,
                ContractContainer::new(&contract_name, entry_points_caller)
            );
    }

    fn contract_env(&self) -> ContractEnv {
        (*self.contract_env).clone()
    }

    fn gas_report(&self) -> GasReport {
        println!("Gas report is unavailable for livenet");
        todo!()
    }

    fn last_call_gas_cost(&self) -> u64 {
        // Todo: implement
        0
    }

    fn sign_message(&self, message: &Bytes, address: &Address) -> Bytes {
        self.casper_client
            .borrow()
            .sign_message(message, address)
            .unwrap()
    }

    fn public_key(&self, address: &Address) -> PublicKey {
        self.casper_client.borrow().address_public_key(address)
    }

    fn transfer(&self, to: Address, amount: U512) -> OdraResult<()> {
        let timestamp = Timestamp::now();
        let client = self.casper_client.borrow_mut();
        client
            .transfer(to, amount, timestamp)
            .map(|_| ())
            .map_err(|e| self.error_msg_to_odra_error(e.error_message(), None))
    }
}

impl LivenetHost {
    /// Decodes the error of a transaction, sent to the contract at `address` if any. A user error
    /// is named after the schema of that contract.
    fn error_msg_to_odra_error(&self, error_msg: String, address: Option<&Address>) -> OdraError {
        let register = self
            .contract_register
            .read()
            .expect("Couldn't read contract register.");
        let contract_name = address.and_then(|address| register.get_name(address));
        match error::find(&error_msg, contract_name) {
            Ok(err) => err,
            _ => OdraError::VmError(VmError::Other(error_msg))
        }
    }
}

/// The error of a failed deployment of the contract `contract_name`. A revert of its constructor
/// or upgrade, or running out of gas, is decoded like the error of an entry point call; any other
/// failure did not get to run the contract and is a deployment error.
fn deployment_error(e: LivenetError, contract_name: &str) -> OdraError {
    if let LivenetError::ExecutionError(error_msg) = &e {
        if let Ok(error) = error::find(error_msg, Some(contract_name)) {
            return error;
        }
    }
    ExecutionError::ContractDeploymentError(e.to_string()).into()
}

/// A read could not be served by the node. `None` would be mistaken for "value not set" by the
/// contract code, so stop with the real reason instead.
pub(crate) fn read_failed(what: &str, address: &Address, e: LivenetError) -> ! {
    panic!(
        "Livenet: reading {what} of {} failed: {}",
        address.to_formatted_string(),
        e.error_message()
    )
}

#[cfg(test)]
mod tests {
    use super::{deployment_error, LivenetHost};
    use odra_casper_rpc_client::casper_client::configuration::CasperClientConfiguration;
    use odra_casper_rpc_client::error::LivenetError;
    use odra_core::casper_types::bytesrepr::{Bytes, ToBytes};
    use odra_core::casper_types::contracts::ContractPackageHash;
    use odra_core::casper_types::{RuntimeArgs, SecretKey};
    use odra_core::entry_point_callback::{EntryPoint, EntryPointsCaller};
    use odra_core::host::{HostContext, HostEnv};
    use odra_core::prelude::*;
    use odra_core::{CallDef, ContractEnv, VmError};
    use std::process::Command;

    const SCENARIO_ENV: &str = "ODRA_LIVENET_PANIC_SCENARIO";
    const ADDRESS: Address = Address::Contract(ContractPackageHash::new([1; 32]));

    #[test]
    fn registered_contract_does_not_keep_the_host_alive() {
        let (host, env) = setup();
        call(&env, "nested_probe").unwrap();
        let weak = Rc::downgrade(&host);
        drop((host, env));
        assert!(
            weak.upgrade().is_none(),
            "the host outlived its environment"
        );
    }

    #[test]
    fn getter_is_called_by_the_account() {
        let (host, env) = setup();
        let account = env.caller();
        assert_eq!(call(&env, "probe"), Ok((account, vec![account, ADDRESS])));
        assert!(host.callstack.borrow().is_empty());
    }

    #[test]
    fn nested_getter_is_called_by_the_contract() {
        let (host, env) = setup();
        let account = env.caller();
        assert_eq!(
            call(&env, "nested_probe"),
            Ok((ADDRESS, vec![account, ADDRESS, ADDRESS]))
        );
        assert!(host.callstack.borrow().is_empty());
    }

    #[test]
    fn reverting_getter_returns_the_error() {
        let (host, env) = setup();
        let account = env.caller();
        assert_eq!(call(&env, "fail"), Err(OdraError::user(7, "Boom")));
        assert!(host.callstack.borrow().is_empty());
        assert_eq!(call(&env, "probe"), Ok((account, vec![account, ADDRESS])));
    }

    #[test]
    fn revert_in_a_nested_getter_returns_the_error() {
        let (host, env) = setup();
        let account = env.caller();
        assert_eq!(call(&env, "nested_fail"), Err(OdraError::user(7, "Boom")));
        assert!(host.callstack.borrow().is_empty());
        assert_eq!(
            call(&env, "nested_probe"),
            Ok((ADDRESS, vec![account, ADDRESS, ADDRESS]))
        );
    }

    #[test]
    fn panicking_getter_returns_a_vm_error() {
        let (host, env) = setup();
        let account = env.caller();
        assert_eq!(call(&env, "panic"), Err(OdraError::VmError(VmError::Panic)));
        assert!(host.callstack.borrow().is_empty());
        assert_eq!(call(&env, "probe"), Ok((account, vec![account, ADDRESS])));
    }

    #[test]
    fn unknown_getter_returns_an_error() {
        let (host, env) = setup();
        assert_eq!(
            call(&env, "nested_unknown"),
            Err(OdraError::VmError(VmError::NoSuchMethod("unknown".into())))
        );
        assert!(host.callstack.borrow().is_empty());
    }

    #[test]
    fn reverted_deployment_returns_the_contract_error() {
        let revert = LivenetError::ExecutionError(String::from("User error: 1"));
        assert_eq!(
            deployment_error(revert, "Second"),
            OdraError::user(1, "SecondError")
        );
        let out_of_gas = LivenetError::ExecutionError(String::from("Out of gas error"));
        assert_eq!(
            deployment_error(out_of_gas, "Second"),
            ExecutionError::OutOfGas.into()
        );
    }

    #[test]
    fn failed_deployment_is_a_deployment_error() {
        let deployment_failed = |e: LivenetError| {
            let message = e.to_string();
            assert_eq!(
                deployment_error(e, "Second"),
                ExecutionError::ContractDeploymentError(message).into()
            );
        };
        deployment_failed(LivenetError::GasNotSet);
        deployment_failed(LivenetError::RpcCommunicationFailure);
        deployment_failed(LivenetError::ExecutionError(String::from(
            "Missing required argument: odra_cfg_package_hash_key_name"
        )));
        deployment_failed(LivenetError::ExecutionError(String::from(
            "Casper Engine error"
        )));
    }

    #[test]
    fn revert_does_not_print_a_panic() {
        let stderr = run_child("revert");
        assert!(!stderr.contains("panicked at"), "{stderr}");
    }

    #[test]
    fn other_panic_prints_the_standard_message() {
        let stderr = run_child("panic");
        assert!(stderr.contains("panicked at"), "{stderr}");
        assert!(stderr.contains("boom"), "{stderr}");
    }

    /// Runs a scenario when started by [run_child], does nothing in a regular test run.
    #[test]
    fn child() {
        let Ok(scenario) = std::env::var(SCENARIO_ENV) else {
            return;
        };
        let (_, env) = setup();
        match scenario.as_str() {
            "revert" => assert!(call(&env, "nested_fail").is_err()),
            "panic" => assert!(call(&env, "panic").is_err()),
            _ => panic!("unknown scenario {scenario}")
        }
    }

    fn run_child(scenario: &str) -> String {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "livenet_host::tests::child", "--nocapture"])
            .env(SCENARIO_ENV, scenario)
            .env_remove("RUST_BACKTRACE")
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8_lossy(&output.stderr).into_owned()
    }

    /// A host with one registered contract; nothing in the scenarios reaches the node.
    fn setup() -> (Rc<LivenetHost>, HostEnv) {
        let configuration = CasperClientConfiguration {
            node_address: String::from("http://localhost:1"),
            events_url: String::from("http://localhost:1/events"),
            chain_name: String::from("casper-test"),
            secret_keys: vec![SecretKey::ed25519_from_bytes([1; 32]).unwrap()],
            secret_key_paths: vec![],
            cspr_cloud_auth_token: None,
            gas_price_tolerance: 1,
            ttl: 60,
            state_root_hash: None
        };
        let host = Rc::new(LivenetHost::with_configuration(configuration));
        let env = HostEnv::new(host.clone());
        let entry_points = [
            "probe",
            "fail",
            "panic",
            "nested_probe",
            "nested_fail",
            "nested_unknown"
        ]
        .into_iter()
        .map(|name| EntryPoint::new(String::from(name), vec![]))
        .collect();
        let caller = EntryPointsCaller::new(entry_points, entry_point);
        host.register_contract(ADDRESS, String::from("Contract"), caller);
        (host, env)
    }

    /// The caller and the call stack seen by the called contract.
    type Probe = (Address, Vec<Address>);

    fn call(env: &HostEnv, entry_point: &str) -> OdraResult<Probe> {
        env.call_contract(
            ADDRESS,
            CallDef::new(entry_point, false, RuntimeArgs::new())
        )
    }

    fn entry_point(env: ContractEnv, call_def: CallDef) -> OdraResult<Bytes> {
        let nested = |entry_point: &str| {
            env.call_contract::<Probe>(
                env.self_address(),
                CallDef::new(entry_point, false, RuntimeArgs::new())
            )
        };
        let result: Probe = match call_def.entry_point() {
            "fail" => env.revert(OdraError::user(7, "Boom")),
            "panic" => panic!("boom"),
            "nested_probe" => nested("probe"),
            "nested_fail" => nested("fail"),
            "nested_unknown" => nested("unknown"),
            _ => (env.caller(), env.call_stack())
        };
        Ok(Bytes::from(result.to_bytes().unwrap()))
    }
}
