use std::cell::RefCell;

use cosmwasm_std::{Attribute, Env, Event, MessageInfo, Response, WasmMsg, SubMsg};
use odra_cosmos_types::{Address, BlockTime};

#[derive(Default)]
pub struct Runtime {
    env: Option<Env>,
    info: Option<MessageInfo>,
    response: Option<Response>
}

impl Runtime {
    pub fn new(env: Env, info: MessageInfo) -> Self {
        Runtime {
            env: Some(env),
            info: Some(info),
            response: Some(Response::new())
        }
    }

    pub fn query(env: Env) -> Self {
        Runtime {
            env: Some(env),
            info: None,
            response: None
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

    pub fn add_event(&mut self, event: Event) {
        if let Some(response) = self.response.as_mut() {
            response.events.push(event);
        }
    }

    pub fn add_call(&mut self, call: WasmMsg) {
        if let Some(response) = self.response.as_mut() {
            response.messages.push(SubMsg::new(call));
        }
    }

    pub fn add_attribute(&mut self, key: String, value: String) {
        if let Some(response) = self.response.as_mut() {
            response.attributes.push(Attribute::new(key, value))
        }
    }

    pub fn get_response(&self) -> Response {
        self.response.as_ref().unwrap().clone()
    }
}

thread_local! {
    pub static RT: RefCell<Runtime> = RefCell::new(Runtime::default());
}
