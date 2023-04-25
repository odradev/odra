use crate::erc1155_receiver::events::{BatchReceived, SingleReceived};
use odra::types::{event::OdraEvent, Address, Bytes, U256};

/// The ERC1155 receiver implementation.
#[odra::module(events = [SingleReceived, BatchReceived])]
pub struct Erc1155Receiver {}

impl Erc1155Receiver {
    pub fn on_erc1155_received(
        &mut self,
        #[allow(unused_variables)] operator: &Address,
        #[allow(unused_variables)] from: &Address,
        #[allow(unused_variables)] token_id: &U256,
        #[allow(unused_variables)] amount: &U256,
        #[allow(unused_variables)] data: &Option<Bytes>
    ) -> bool {
        SingleReceived {
            operator: Some(*operator),
            from: Some(*from),
            token_id: *token_id,
            amount: *amount,
            data: data.clone()
        }
        .emit();
        true
    }
    pub fn on_erc1155_batch_received(
        &mut self,
        #[allow(unused_variables)] operator: &Address,
        #[allow(unused_variables)] from: &Address,
        #[allow(unused_variables)] token_ids: &Vec<U256>,
        #[allow(unused_variables)] amounts: &Vec<U256>,
        #[allow(unused_variables)] data: &Option<Bytes>
    ) -> bool {
        BatchReceived {
            operator: Some(*operator),
            from: Some(*from),
            token_ids: token_ids.clone(),
            amounts: amounts.clone(),
            data: data.clone()
        }
        .emit();
        true
    }
}
#[cfg(feature = "casper")]
impl odra::types::contract_def::HasIdent for Erc1155Receiver {
    fn ident() -> String {
        String::from("Erc1155Receiver")
    }
}
#[cfg(feature = "casper")]
impl odra::types::contract_def::HasEntrypoints for Erc1155Receiver {
    fn entrypoints() -> Vec<odra::types::contract_def::Entrypoint> {
        vec![
            odra::types::contract_def::Entrypoint {
                ident: String::from("on_erc1155_received"),
                args: vec![
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(operator)),
                        ty: <Address as odra::types::Typed>::ty()
                    },
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(from)),
                        ty: <Address as odra::types::Typed>::ty()
                    },
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(token_id)),
                        ty: <U256 as odra::types::Typed>::ty()
                    },
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(amount)),
                        ty: <U256 as odra::types::Typed>::ty()
                    },
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(data)),
                        ty: <Option<Bytes> as odra::types::Typed>::ty()
                    },
                ],
                is_mut: true,
                ret: <bool as odra::types::Typed>::ty(),
                ty: odra::types::contract_def::EntrypointType::Public
            },
            odra::types::contract_def::Entrypoint {
                ident: String::from("on_erc1155_batch_received"),
                args: vec![
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(operator)),
                        ty: <Address as odra::types::Typed>::ty()
                    },
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(from)),
                        ty: <Address as odra::types::Typed>::ty()
                    },
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(token_ids)),
                        ty: <Vec<U256> as odra::types::Typed>::ty()
                    },
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(amounts)),
                        ty: <Vec<U256> as odra::types::Typed>::ty()
                    },
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(data)),
                        ty: <Option<Bytes> as odra::types::Typed>::ty()
                    },
                ],
                is_mut: true,
                ret: <bool as odra::types::Typed>::ty(),
                ty: odra::types::contract_def::EntrypointType::Public
            },
        ]
    }
}
pub struct Erc1155ReceiverDeployer;

#[cfg(all(feature = "casper", not(target_arch = "wasm32")))]
impl Erc1155ReceiverDeployer {
    pub fn default() -> Erc1155ReceiverRef {
        let address =
            odra::test_env::register_contract(&"erc1155_receiver", odra::types::CallArgs::new());
        Erc1155ReceiverRef::at(address)
    }
}
#[cfg(feature = "mock-vm")]
impl Erc1155ReceiverDeployer {
    pub fn default() -> Erc1155ReceiverRef {
        use odra::types::CallArgs;
        use std::collections::HashMap;
        let mut entrypoints =
            HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Vec<u8>)>::new();
        entrypoints.insert(
            stringify!(on_erc1155_received).to_string(),
            (
                {
                    let mut args: Vec<String> = vec![];
                    args.push(stringify!(operator).to_string());
                    args.push(stringify!(from).to_string());
                    args.push(stringify!(token_id).to_string());
                    args.push(stringify!(amount).to_string());
                    args.push(stringify!(data).to_string());
                    args
                },
                |name, args| {
                    if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                        odra::contract_env::revert(odra::types::ExecutionError::non_payable());
                    }
                    let mut instance = <Erc1155Receiver as odra::Instance>::instance(name.as_str());
                    let result = instance.on_erc1155_received(
                        &args.get(stringify!(operator)),
                        &args.get(stringify!(from)),
                        &args.get(stringify!(token_id)),
                        &args.get(stringify!(amount)),
                        &args.get(stringify!(data))
                    );
                    odra::types::MockVMType::ser(&result).unwrap()
                }
            )
        );
        entrypoints.insert(
            stringify!(on_erc1155_batch_received).to_string(),
            (
                {
                    let mut args: Vec<String> = vec![];
                    args.push(stringify!(operator).to_string());
                    args.push(stringify!(from).to_string());
                    args.push(stringify!(token_ids).to_string());
                    args.push(stringify!(amounts).to_string());
                    args.push(stringify!(data).to_string());
                    args
                },
                |name, args| {
                    if odra::contract_env::attached_value() > odra::types::Balance::zero() {
                        odra::contract_env::revert(odra::types::ExecutionError::non_payable());
                    }
                    let mut instance = <Erc1155Receiver as odra::Instance>::instance(name.as_str());
                    let result = instance.on_erc1155_batch_received(
                        &args.get(stringify!(operator)),
                        &args.get(stringify!(from)),
                        &args.get(stringify!(token_ids)),
                        &args.get(stringify!(amounts)),
                        &args.get(stringify!(data))
                    );
                    odra::types::MockVMType::ser(&result).unwrap()
                }
            )
        );
        let mut constructors =
            HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Vec<u8>)>::new();
        let address = odra::test_env::register_contract(None, constructors, entrypoints);
        Erc1155ReceiverRef::at(address)
    }
}
#[cfg(feature = "casper-livenet")]
impl Erc1155ReceiverDeployer {
    pub fn register(address: odra::types::Address) -> Erc1155ReceiverRef {
        use odra::types::CallArgs;
        use std::collections::HashMap;
        let mut entrypoints =
            HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Vec<u8>)>::new();
        entrypoints.insert(
            stringify!(on_erc1155_received).to_string(),
            (
                {
                    let mut args: Vec<String> = vec![];
                    args.push(stringify!(operator).to_string());
                    args.push(stringify!(from).to_string());
                    args.push(stringify!(token_id).to_string());
                    args.push(stringify!(amount).to_string());
                    args.push(stringify!(data).to_string());
                    args
                },
                |name, args| {
                    let mut instance = <Erc1155Receiver as odra::Instance>::instance("contract");
                    let result = instance.on_erc1155_received(
                        args.get(stringify!(operator))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(from))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(token_id))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(amount))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(data))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap()
                    );
                    let clvalue = odra::casper::casper_types::CLValue::from_t(result).unwrap();
                    odra::casper::casper_types::bytesrepr::ToBytes::into_bytes(clvalue).unwrap()
                }
            )
        );
        entrypoints.insert(
            stringify!(on_erc1155_batch_received).to_string(),
            (
                {
                    let mut args: Vec<String> = vec![];
                    args.push(stringify!(operator).to_string());
                    args.push(stringify!(from).to_string());
                    args.push(stringify!(token_ids).to_string());
                    args.push(stringify!(amounts).to_string());
                    args.push(stringify!(data).to_string());
                    args
                },
                |name, args| {
                    let mut instance = <Erc1155Receiver as odra::Instance>::instance("contract");
                    let result = instance.on_erc1155_batch_received(
                        args.get(stringify!(operator))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(from))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(token_ids))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(amounts))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(data))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap()
                    );
                    let clvalue = odra::casper::casper_types::CLValue::from_t(result).unwrap();
                    odra::casper::casper_types::bytesrepr::ToBytes::into_bytes(clvalue).unwrap()
                }
            )
        );
        odra::client_env::register_existing_contract(address, entrypoints);
        Erc1155ReceiverRef::at(address)
    }
    pub fn default() -> Erc1155ReceiverRef {
        use odra::types::CallArgs;
        use std::collections::HashMap;
        let mut entrypoints =
            HashMap::<String, (Vec<String>, fn(String, CallArgs) -> Vec<u8>)>::new();
        entrypoints.insert(
            stringify!(on_erc1155_received).to_string(),
            (
                {
                    let mut args: Vec<String> = vec![];
                    args.push(stringify!(operator).to_string());
                    args.push(stringify!(from).to_string());
                    args.push(stringify!(token_id).to_string());
                    args.push(stringify!(amount).to_string());
                    args.push(stringify!(data).to_string());
                    args
                },
                |name, args| {
                    let mut instance = <Erc1155Receiver as odra::Instance>::instance("contract");
                    let result = instance.on_erc1155_received(
                        args.get(stringify!(operator))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(from))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(token_id))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(amount))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(data))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap()
                    );
                    let clvalue = odra::casper::casper_types::CLValue::from_t(result).unwrap();
                    odra::casper::casper_types::bytesrepr::ToBytes::into_bytes(clvalue).unwrap()
                }
            )
        );
        entrypoints.insert(
            stringify!(on_erc1155_batch_received).to_string(),
            (
                {
                    let mut args: Vec<String> = vec![];
                    args.push(stringify!(operator).to_string());
                    args.push(stringify!(from).to_string());
                    args.push(stringify!(token_ids).to_string());
                    args.push(stringify!(amounts).to_string());
                    args.push(stringify!(data).to_string());
                    args
                },
                |name, args| {
                    let mut instance = <Erc1155Receiver as odra::Instance>::instance("contract");
                    let result = instance.on_erc1155_batch_received(
                        args.get(stringify!(operator))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(from))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(token_ids))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(amounts))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap(),
                        args.get(stringify!(data))
                            .cloned()
                            .unwrap()
                            .into_t()
                            .unwrap()
                    );
                    let clvalue = odra::casper::casper_types::CLValue::from_t(result).unwrap();
                    odra::casper::casper_types::bytesrepr::ToBytes::into_bytes(clvalue).unwrap()
                }
            )
        );
        let address = odra::client_env::deploy_new_contract(
            &"erc1155_receiver",
            odra::types::CallArgs::new(),
            entrypoints
        );
        Erc1155ReceiverRef::at(address)
    }
}
pub struct Erc1155ReceiverRef {
    address: odra::types::Address,
    attached_value: Option<odra::types::Balance>
}
impl Erc1155ReceiverRef {
    pub fn at(address: odra::types::Address) -> Self {
        Self {
            address,
            attached_value: None
        }
    }
    pub fn address(&self) -> odra::types::Address {
        self.address.clone()
    }
    pub fn with_tokens<T>(&self, amount: T) -> Self
    where
        T: Into<odra::types::Balance>
    {
        Self {
            address: self.address,
            attached_value: Some(amount.into())
        }
    }
}
impl Erc1155ReceiverRef {
    pub fn on_erc1155_received(
        &mut self,
        #[allow(unused_variables)] operator: &Address,
        #[allow(unused_variables)] from: &Address,
        #[allow(unused_variables)] token_id: &U256,
        #[allow(unused_variables)] amount: &U256,
        #[allow(unused_variables)] data: &Option<Bytes>
    ) -> bool {
        let args = {
            let mut args = odra::types::CallArgs::new();
            args.insert(stringify!(operator), *operator);
            args.insert(stringify!(from), *from);
            args.insert(stringify!(token_id), *token_id);
            args.insert(stringify!(amount), *amount);
            args.insert(stringify!(data), data.clone());
            args
        };
        odra::call_contract(
            self.address,
            "on_erc1155_received",
            args,
            self.attached_value
        )
    }
    pub fn on_erc1155_batch_received(
        &mut self,
        #[allow(unused_variables)] operator: &Address,
        #[allow(unused_variables)] from: &Address,
        #[allow(unused_variables)] token_ids: &Vec<U256>,
        #[allow(unused_variables)] amounts: &Vec<U256>,
        #[allow(unused_variables)] data: &Option<Bytes>
    ) -> bool {
        let args = {
            let mut args = odra::types::CallArgs::new();
            args.insert(stringify!(operator), operator.clone());
            args.insert(stringify!(from), from.clone());
            args.insert(stringify!(token_ids), token_ids.clone());
            args.insert(stringify!(amounts), amounts.clone());
            args.insert(stringify!(data), data.clone());
            args
        };
        odra::call_contract(
            self.address,
            "on_erc1155_batch_received",
            args,
            self.attached_value
        )
    }
}

pub mod events {
    use odra::types::{Address, Bytes, U256};

    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct SingleReceived {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub token_id: U256,
        pub amount: U256,
        pub data: Option<Bytes>
    }

    #[derive(odra::Event, PartialEq, Eq, Debug, Clone)]
    pub struct BatchReceived {
        pub operator: Option<Address>,
        pub from: Option<Address>,
        pub token_ids: Vec<U256>,
        pub amounts: Vec<U256>,
        pub data: Option<Bytes>
    }
}
