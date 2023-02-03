use std::cell::RefCell;

use cosmwasm_std::{Env, MessageInfo};
use odra_cosmos_types::{Address, BlockTime};

#[derive(Default)]
pub struct Runtime {
    env: Option<Env>,
    info: Option<MessageInfo>
}

impl Runtime {
    pub fn new(env: Env, info: MessageInfo) -> Self {
        Runtime {
            env: Some(env),
            info: Some(info)
        }
    }

    pub fn query(env: Env) -> Self {
        Runtime {
            env: Some(env),
            info: None
        }
    }

    pub fn block_time(&self) -> BlockTime {
        self.env.as_ref().unwrap().block.time.seconds()
    }

    pub fn caller(&self) -> Address {
        let sender = self.info.as_ref().unwrap().sender.clone();
        Address::new(sender.as_bytes())
    }

    pub fn contract_address(&self) -> Address {
        let address = self.env.as_ref().unwrap().contract.address.clone();
        Address::new(address.as_bytes())
    }
}

thread_local! {
    pub static RT: RefCell<Runtime> = RefCell::new(Runtime::default());
}
