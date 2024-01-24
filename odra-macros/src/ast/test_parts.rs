use derive_try_from_ref::TryFromRef;
use syn::parse_quote;

use crate::{ir::ModuleImplIR, utils};

use super::{
    deployer_item::DeployerItem,
    host_ref_item::HostRefItem,
    parts_utils::{UsePreludeItem, UseSuperItem}
};

#[derive(syn_derive::ToTokens)]
pub struct TestPartsReexportItem {
    not_wasm_attr: syn::Attribute,
    reexport_stmt: syn::Stmt
}

impl TryFrom<&'_ ModuleImplIR> for TestPartsReexportItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let test_parts_ident = module.test_parts_mod_ident()?;
        Ok(Self {
            not_wasm_attr: utils::attr::not_wasm32(),
            reexport_stmt: parse_quote!(pub use #test_parts_ident::*;)
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct PartsModuleItem {
    attr: syn::Attribute,
    mod_token: syn::token::Mod,
    ident: syn::Ident
}

impl TryFrom<&'_ ModuleImplIR> for PartsModuleItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            attr: utils::attr::not_wasm32(),
            mod_token: Default::default(),
            ident: module.test_parts_mod_ident()?
        })
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct TestPartsItem {
    parts_module: PartsModuleItem,
    #[syn(braced)]
    #[default]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    #[expr(UseSuperItem)]
    use_super: UseSuperItem,
    #[syn(in = brace_token)]
    #[expr(UsePreludeItem)]
    use_prelude: UsePreludeItem,
    #[syn(in = brace_token)]
    host_ref: HostRefItem,
    #[syn(in = brace_token)]
    deployer: DeployerItem
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{self, mock};

    #[test]
    fn test_parts() {
        let module = mock::module_impl();
        let actual = TestPartsItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[cfg(not(target_arch = "wasm32"))]
            mod __erc20_test_parts {
                use super::*;
                use odra::prelude::*;

                pub struct Erc20HostRef {
                    address: odra::Address,
                    env: odra::HostEnv,
                    attached_value: odra::U512
                }

                impl Erc20HostRef {
                    pub fn new(address: odra::Address, env: odra::HostEnv) -> Self {
                        Self {
                            address,
                            env,
                            attached_value: Default::default()
                        }
                    }

                    pub fn with_tokens(&self, tokens: odra::U512) -> Self {
                        Self {
                            address: self.address,
                            env: self.env.clone(),
                            attached_value: tokens
                        }
                    }

                    pub fn address(&self) -> &odra::Address {
                        &self.address
                    }

                    pub fn env(&self) -> &odra::HostEnv {
                        &self.env
                    }

                    pub fn get_event<T>(&self, index: i32) -> Result<T, odra::event::EventError>
                    where
                        T: odra::FromBytes + odra::casper_event_standard::EventInstance,
                    {
                        self.env.get_event(&self.address, index)
                    }

                    pub fn last_call(&self) -> odra::ContractCallResult {
                        self.env.last_call().contract_last_call(self.address)
                    }

                    pub fn try_total_supply(&self) -> Result<U256, odra::OdraError> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                String::from("total_supply"),
                                false,
                                {
                                    let mut named_args = odra::RuntimeArgs::new();
                                    if self.attached_value > odra::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }

                    pub fn total_supply(&self) -> U256 {
                        self.try_total_supply().unwrap()
                    }

                    pub fn try_pay_to_mint(&mut self) -> Result<(), odra::OdraError> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                String::from("pay_to_mint"),
                                true,
                                {
                                    let mut named_args = odra::RuntimeArgs::new();
                                    if self.attached_value > odra::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }

                    pub fn pay_to_mint(&mut self) {
                        self.try_pay_to_mint().unwrap()
                    }

                    pub fn try_approve(&mut self, to: Address, amount: U256) -> Result<(), odra::OdraError> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                String::from("approve"),
                                true,
                                {
                                    let mut named_args = odra::RuntimeArgs::new();
                                    if self.attached_value > odra::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    let _ = named_args.insert("to", to);
                                    let _ = named_args.insert("amount", amount);
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }

                    pub fn approve(&mut self, to: Address, amount: U256) {
                        self.try_approve(to, amount).unwrap()
                    }

                    pub fn try_airdrop(&self, to: odra::prelude::vec::Vec<Address>, amount: U256) -> Result<(), odra::OdraError> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                String::from("airdrop"),
                                false,
                                {
                                    let mut named_args = odra::RuntimeArgs::new();
                                    if self.attached_value > odra::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    let _ = named_args.insert("to", to);
                                    let _ = named_args.insert("amount", amount);
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }

                    pub fn airdrop(&self, to: odra::prelude::vec::Vec<Address>, amount: U256) {
                        self.try_airdrop(to, amount).unwrap()
                    }
                }

                pub struct Erc20Deployer;

                impl Erc20Deployer {
                    pub fn init(env: &odra::HostEnv, total_supply: Option<U256>) -> Erc20HostRef {
                        let entry_points = odra::prelude::vec![
                            odra::EntryPoint::new(odra::prelude::string::String::from("init"), odra::prelude::vec![
                                odra::EntryPointArgument::new(odra::prelude::string::String::from("total_supply"), <Option::<U256> as odra::casper_types::CLTyped>::cl_type())
                            ]),
                            odra::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                            odra::EntryPoint::new(odra::prelude::string::String::from("pay_to_mint"), odra::prelude::vec![]),
                            odra::EntryPoint::new(odra::prelude::string::String::from("approve"), odra::prelude::vec![
                                odra::EntryPointArgument::new(odra::prelude::string::String::from("to"), <Address as odra::casper_types::CLTyped>::cl_type()),
                                odra::EntryPointArgument::new(odra::prelude::string::String::from("amount"), <U256 as odra::casper_types::CLTyped>::cl_type())
                            ]),
                            odra::EntryPoint::new(odra::prelude::string::String::from("airdrop"), odra::prelude::vec![
                                odra::EntryPointArgument::new(odra::prelude::string::String::from("to"), <odra::prelude::vec::Vec<Address> as odra::casper_types::CLTyped>::cl_type()),
                                odra::EntryPointArgument::new(odra::prelude::string::String::from("amount"), <U256 as odra::casper_types::CLTyped>::cl_type())
                            ])
                        ];
                        let caller = odra::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                            match call_def.method() {
                                "init" => {
                                    let result = __erc20_exec_parts::execute_init(contract_env);
                                    odra::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                                }
                                "total_supply" => {
                                    let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                    odra::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                                }
                                "pay_to_mint" => {
                                    let result = __erc20_exec_parts::execute_pay_to_mint(contract_env);
                                    odra::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                                }
                                "approve" => {
                                    let result = __erc20_exec_parts::execute_approve(contract_env);
                                    odra::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                                }
                                "airdrop" => {
                                    let result = __erc20_exec_parts::execute_airdrop(contract_env);
                                    odra::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                                }
                                name => Err(odra::OdraError::VmError(
                                    odra::VmError::NoSuchMethod(odra::prelude::String::from(name))
                                ))
                            }
                        });

                        let address = env.new_contract(
                            "Erc20",
                            Some({
                                let mut named_args = odra::RuntimeArgs::new();
                                let _ = named_args.insert("total_supply", total_supply);
                                named_args
                            }),
                            Some(caller)
                        );
                        Erc20HostRef {
                            address,
                            env: env.clone(),
                            attached_value: odra::U512::zero()
                        }
                    }
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn test_trait_impl_parts() {
        let module = mock::module_trait_impl();
        let actual = TestPartsItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[cfg(not(target_arch = "wasm32"))]
            mod __erc20_test_parts {
                use super::*;
                use odra::prelude::*;

                pub struct Erc20HostRef {
                    address: odra::Address,
                    env: odra::HostEnv,
                    attached_value: odra::U512
                }

                impl Erc20HostRef {
                    pub fn new(address: odra::Address, env: odra::HostEnv) -> Self {
                        Self {
                            address,
                            env,
                            attached_value: Default::default()
                        }
                    }

                    pub fn with_tokens(&self, tokens: odra::U512) -> Self {
                        Self {
                            address: self.address,
                            env: self.env.clone(),
                            attached_value: tokens
                        }
                    }

                    pub fn address(&self) -> &odra::Address {
                        &self.address
                    }

                    pub fn env(&self) -> &odra::HostEnv {
                        &self.env
                    }

                    pub fn get_event<T>(&self, index: i32) -> Result<T, odra::event::EventError>
                    where
                        T: odra::FromBytes + odra::casper_event_standard::EventInstance,
                    {
                        self.env.get_event(&self.address, index)
                    }

                    pub fn last_call(&self) -> odra::ContractCallResult {
                        self.env.last_call().contract_last_call(self.address)
                    }

                    pub fn try_total_supply(&self) -> Result<U256, odra::OdraError> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                String::from("total_supply"),
                                false,
                                {
                                    let mut named_args = odra::RuntimeArgs::new();
                                    if self.attached_value > odra::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }

                    pub fn total_supply(&self) -> U256 {
                        self.try_total_supply().unwrap()
                    }

                    pub fn try_pay_to_mint(&mut self) -> Result<(), odra::OdraError> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                String::from("pay_to_mint"),
                                true,
                                {
                                    let mut named_args = odra::RuntimeArgs::new();
                                    if self.attached_value > odra::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }

                    pub fn pay_to_mint(&mut self) {
                        self.try_pay_to_mint().unwrap()
                    }
                }

                pub struct Erc20Deployer;

                impl Erc20Deployer {
                    pub fn init(env: &odra::HostEnv) -> Erc20HostRef {
                        let entry_points = odra::prelude::vec![
                            odra::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                            odra::EntryPoint::new(odra::prelude::string::String::from("pay_to_mint"), odra::prelude::vec![])
                        ];
                        let caller = odra::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                            match call_def.method() {
                                "total_supply" => {
                                    let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                    odra::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                                }
                                "pay_to_mint" => {
                                    let result = __erc20_exec_parts::execute_pay_to_mint(contract_env);
                                    odra::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| odra::OdraError::ExecutionError(err.into()))
                                }
                                name => Err(odra::OdraError::VmError(
                                    odra::VmError::NoSuchMethod(odra::prelude::String::from(name)),
                                ))
                            }
                        });

                        let address = env.new_contract(
                            "Erc20",
                            None,
                            Some(caller)
                        );
                        Erc20HostRef {
                            address,
                            env: env.clone(),
                            attached_value: odra::U512::zero()
                        }
                    }
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}
