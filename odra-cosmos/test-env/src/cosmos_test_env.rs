use std::{cell::RefCell, env, fs, path::Path};

use cosmwasm_std::{to_vec, Binary, ContractResult, Env, Event, MessageInfo, Response};
use cosmwasm_vm::{
    call_execute, call_instantiate, call_query,
    testing::{mock_env, mock_info, mock_instance, MockApi, MockQuerier, MockStorage},
    Instance, VmError, VmResult
};

use odra_cosmos_types::{Address, BlockTime, CallArgs, OdraType, Balance};
use odra_types::{ExecutionError, OdraError, VmError as OdraVmError};
use serde_json::{Map, Value};

thread_local! {
    pub static ENV: RefCell<TestEnv> = RefCell::new(TestEnv::new());
}

const ARG_ACTION_NAME: &str = "name";
const ARG_ACTION_ARGS: &str = "args";
const CALL_ARG_CONSTRUCTOR: &str = "constructor";

#[allow(dead_code)]
pub struct TestEnv {
    accounts: Vec<Address>,
    active_account: Address,
    block_time: BlockTime,
    attached_value: Option<Balance>,
    wasm_path: Option<String>,
    mock_instance: Option<Instance<MockApi, MockStorage, MockQuerier>>,
    env: Env,
    events: Vec<Event>,
    error: Option<OdraError>
}

impl TestEnv {
    pub fn new() -> Self {
        let accounts = vec!["Alice", "Bob", "Charles", "David"];
        let accounts = accounts
            .iter()
            .map(|bytes| Address::new(bytes.as_bytes()))
            .collect::<Vec<_>>();
        let active_account = accounts.first().unwrap().clone();

        Self {
            accounts,
            active_account,
            block_time: 0,
            wasm_path: None,
            mock_instance: None,
            attached_value: None,
            env: mock_env(),
            events: vec![],
            error: None
        }
    }

    pub fn deploy_contract(&mut self, wasm_path: &str, constructor: &str, args: CallArgs) {
        self.wasm_path = Some(String::from(wasm_path));
        let wasm = Self::read_wasm_file_bytes(wasm_path);
        self.mock_instance = Some(mock_instance(&wasm, &[]));

        let message_info = self.active_account_message_info();
        let mut instance = self
            .mock_instance
            .as_mut()
            .expect("The instance should initialized");

        let msg = build_message(constructor, args);
        let result: VmResult<ContractResult<Response>> =
            call_instantiate(&mut instance, &self.env, &message_info, &msg);
        self.handle_error(&result);
        self.handle_response(&result);
    }

    pub fn execute(&mut self, entry_point: &str, args: CallArgs) {
        self.error = None;

        let message_info = self.active_account_message_info();
        let msg = build_message(entry_point, args);
        let instance = self
            .mock_instance
            .as_mut()
            .expect("The instance should initialized");

        let result: VmResult<ContractResult<Response>> =
            call_execute(instance, &self.env, &message_info, &msg);

        self.handle_error(&result);
        self.handle_response(&result);

        self.active_account = self.get_account(0);
    }

    pub fn query<T: OdraType>(&mut self, entry_point: &str, args: CallArgs) -> T {
        self.error = None;

        let msg = build_message(entry_point, args);
        let instance = self
            .mock_instance
            .as_mut()
            .expect("The instance should initialized");

        let result: VmResult<ContractResult<Binary>> = call_query(instance, &self.env, &msg);

        self.handle_error(&result);

        self.active_account = self.get_account(0);

        if let Ok(result) = result {
            if let ContractResult::Ok(binary) = result {
                return T::deser(binary.0).unwrap();
            }
        };

        T::deser(vec![]).unwrap()
    }

    pub fn set_caller(&mut self, address: Address) {
        self.active_account = address;
    }

    /// Increases the current value of block_time.
    pub fn advance_block_time_by(&mut self, seconds: BlockTime) {
        self.env.block.time.plus_seconds(seconds);
    }

    /// Sets the value that will be attached to the next contract call.
    pub fn attach_value(&mut self, amount: Balance) {
        self.attached_value = Some(amount);
    }

    /// Get one of the predefined accounts.
    pub fn get_account(&self, n: usize) -> Address {
        *self.accounts.get(n).unwrap()
    }

    /// Returns possible error.
    pub fn get_error(&self) -> Option<OdraError> {
        self.error.clone()
    }

    /// Reads a given compiled contract file based on path
    fn read_wasm_file_bytes<T: AsRef<Path>>(contract_file: T) -> Vec<u8> {
        if contract_file.as_ref().is_relative() {
            // Find first path to a given file found in a list of paths
            let wasm_path = env::current_dir()
                .expect("should get current working dir")
                .join("wasm");
            let mut filename = wasm_path.clone();
            filename.push(contract_file.as_ref());
            if let Ok(wasm_bytes) = fs::read(&filename) {
                return wasm_bytes;
            }
        }
        // Try just opening in case the arg is a valid path relative to current working dir, or is a
        // valid absolute path.
        if let Ok(wasm_bytes) = fs::read(contract_file.as_ref()) {
            return wasm_bytes;
        }
        let error_msg = "\nFailed to open compiled Wasm file.".to_string();

        panic!("{}\n", error_msg);
    }

    fn active_account_message_info(&self) -> MessageInfo {
        let account_name: String = self.active_account.to_string();
        let mut funds = vec![];
        if let Some(value) = self.attached_value {
            funds.push(cosmwasm_std::Coin::new(value.as_u128(), "ucosm"));
        }
        mock_info(&account_name, &funds)
    }

    fn handle_error<S>(&mut self, result: &VmResult<ContractResult<S>>) {
        if let Err(error) = result {
            self.error = Some(parse_error(error));
        };
    }

    fn handle_response(&mut self, result: &VmResult<ContractResult<Response>>) {
        if let Ok(result) = result {
            if let ContractResult::Ok(response) = result {
                let events = response.events.clone();
                self.events.extend(events.into_iter());
            }
        };
    }
}

fn build_message(entry_point: &str, args: CallArgs) -> Vec<u8> {
    let args = args
        .arg_names()
        .iter()
        .filter(|name| *name != CALL_ARG_CONSTRUCTOR)
        .map(|name| args.get_as_value(name))
        .collect::<Vec<Value>>();

    let args = Value::Array(args);
    let mut ep = Map::new();
    ep.insert(
        ARG_ACTION_NAME.to_string(),
        Value::String(entry_point.to_string())
    );
    ep.insert(ARG_ACTION_ARGS.to_string(), args);
    let root = Value::Object(ep);
    to_vec(&root).unwrap()
}

fn parse_error(err: &VmError) -> OdraError {
    match err {
        VmError::RuntimeErr { msg } => {
            // The `msg` looks like "Wasmer runtime error: RuntimeError: Aborted:
            // panicked at '{{original message}}', {{path}}", so search for the string
            // between quotes ''.
            let start = msg.find("'").unwrap();
            let end = msg.rfind("'").unwrap();
            let original_err_msg = &msg[start + 1..end];
            OdraError::ExecutionError(ExecutionError::new(0, original_err_msg))
        }
        other => OdraError::VmError(OdraVmError::Other(format!("{}", other)))
    }
}

#[cfg(test)]
mod tests {
    use super::build_message;
    use odra_cosmos_types::CallArgs;

    #[test]
    fn parsing_args() {
        let ep = "init";
        let mut args = CallArgs::new();
        args.insert("value", 123);
        dbg!(build_message(ep, args));
    }
}
