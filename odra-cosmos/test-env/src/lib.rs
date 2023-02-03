//! TestEnv bindings for Casper's InMemoryWasmTestBuilder.
#[doc(hidden)]
pub mod dummy_contract_env;
pub mod test_env;

use std::{cell::RefCell, env, fs, path::Path};

use convert_case::Casing;
use cosmwasm_std::{to_vec, Env, MessageInfo, Response};
use cosmwasm_vm::{
    call_execute, call_instantiate, call_query,
    testing::{mock_env, mock_info, mock_instance, MockApi, MockQuerier, MockStorage},
    Instance
};

use odra_cosmos_types::{Address, BlockTime, CallArgs, OdraType, U512};
use serde_json::{Map, Value};

thread_local! {
    pub static ENV: RefCell<TestEnv> = RefCell::new(TestEnv::new());
}

#[allow(dead_code)]
pub struct TestEnv {
    accounts: Vec<Address>,
    active_account: Address,
    block_time: BlockTime,
    attached_value: Option<U512>,
    wasm_path: Option<String>,
    mock_instance: Option<Instance<MockApi, MockStorage, MockQuerier>>,
    env: Env
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
            env: mock_env()
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

        let msg = Self::build_message(constructor, args);
        let init_res: Response = call_instantiate(&mut instance, &self.env, &message_info, &msg)
            .unwrap()
            .unwrap();

        println!("{:?}", init_res);
    }

    fn build_message(entry_point: &str, args: CallArgs) -> Vec<u8> {
        let mut args_map = Map::new();
        args.arg_names().iter().for_each(|name| {
            args_map.insert(name.to_owned(), args.get_as_value(name));
        });

        let args = Value::Object(args_map);
        let mut ep = Map::new();
        ep.insert(
            entry_point.to_case(convert_case::Case::Pascal).to_string(),
            args
        );
        let root = Value::Object(ep);

        to_vec(&root).unwrap()
    }

    pub fn execute(&mut self, entry_point: &str, args: CallArgs) {
        let message_info = self.active_account_message_info();
        let msg = Self::build_message(entry_point, args);
        let response: Response = call_execute(
            self.mock_instance.as_mut().unwrap(),
            &self.env,
            &message_info,
            &msg
        )
        .unwrap()
        .unwrap();

        dbg!(response);
    }

    pub fn query<T: OdraType>(&mut self, entry_point: &str, args: CallArgs) -> T {
        let msg = Self::build_message(entry_point, args);

        let res = call_query(self.mock_instance.as_mut().unwrap(), &self.env, &msg)
            .unwrap()
            .unwrap();

        let data = res.0;

        T::deser(data).unwrap()
    }

    /// Increases the current value of block_time.
    pub fn advance_block_time_by(&mut self, seconds: BlockTime) {
        self.env.block.time.plus_seconds(seconds);
    }

    /// Sets the value that will be attached to the next contract call.
    pub fn attach_value(&mut self, amount: U512) {
        self.attached_value = Some(amount);
    }

    // /// Returns the balance of the given account.
    // ///
    // /// The accepted value can be either an [Address::Account] or [Address::Contract].
    // pub fn token_balance(&self, address: Address) -> U512 {
    //     match address {
    //         Address::Account(account_hash) => self.get_account_cspr_balance(account_hash),
    //         Address::Contract(contract_hash) => self.get_contract_cspr_balance(contract_hash)
    //     }
    // }

    /// Reads a given compiled contract file based on path
    fn read_wasm_file_bytes<T: AsRef<Path>>(contract_file: T) -> Vec<u8> {
        // let mut attempted_paths = vec![];

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
        let account_name: String = self.active_account.into();
        let mut funds = vec![];
        if let Some(value) = self.attached_value {
            funds.push(cosmwasm_std::Coin::new(value.as_u128(), "ucosm"));
        }
        mock_info(&account_name, &funds)
    }
}
