use derive_try_from_ref::TryFromRef;
use syn::parse_quote;

use crate::{ir::ModuleImplIR, utils};

use super::{
    deployer_item::DeployerItem,
    host_ref_item::{HasIdentTraitImplItem, HostRefItem},
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
    trait_has_ident_impl_item: HasIdentTraitImplItem,
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

                /// [Erc20] Host Ref.
                pub struct Erc20HostRef {
                    address: Address,
                    env: odra::host::HostEnv,
                    attached_value: odra::casper_types::U512
                }

                impl odra::host::HostRef for Erc20HostRef {
                    fn new(address: Address, env: odra::host::HostEnv) -> Self {
                        Self {
                            address,
                            env,
                            attached_value: Default::default()
                        }
                    }

                    fn with_tokens(&self, tokens: odra::casper_types::U512) -> Self {
                        Self {
                            address: self.address,
                            env: self.env.clone(),
                            attached_value: tokens
                        }
                    }

                    fn address(&self) -> &Address {
                        &self.address
                    }

                    fn env(&self) -> &odra::host::HostEnv {
                        &self.env
                    }

                    fn get_event<T>(&self, index: i32) -> Result<T, odra::EventError>
                    where
                        T: odra::casper_types::bytesrepr::FromBytes + odra::casper_event_standard::EventInstance,
                    {
                        self.env.get_event(self, index)
                    }

                    fn last_call(&self) -> odra::ContractCallResult {
                        self.env.last_call_result(self.address)
                    }
                }

                impl Erc20HostRef {
                    /// Initializes the contract with the given parameters.
                    pub fn init(&mut self, total_supply: Option<U256>) {
                        self.try_init(total_supply).unwrap()
                    }

                    /// Returns the total supply of the token.
                    pub fn total_supply(&self) -> U256 {
                        self.try_total_supply().unwrap()
                    }

                    /// Pay to mint.
                    pub fn pay_to_mint(&mut self) {
                        self.try_pay_to_mint().unwrap()
                    }

                    /// Approve.
                    pub fn approve(&mut self, to: &Address, amount: &U256, msg: Maybe<String>) {
                        self.try_approve(to, amount, msg).unwrap()
                    }

                    /// Airdrops the given amount to the given addresses.
                    pub fn airdrop(&self, to: &[Address], amount: &U256) {
                        self.try_airdrop(to, amount).unwrap()
                    }
                }

                impl Erc20HostRef {
                    /// Initializes the contract with the given parameters.
                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_init(&mut self, total_supply: Option<U256>) -> OdraResult<()> {
                        self.env
                            .call_contract(
                                self.address,
                                odra::CallDef::new(
                                    odra::prelude::string::String::from("init"),
                                    true,
                                    {
                                        let mut named_args = odra::casper_types::RuntimeArgs::new();
                                        if self.attached_value > odra::casper_types::U512::zero() {
                                            let _ = named_args.insert("amount", self.attached_value);
                                        }
                                        odra::args::EntrypointArgument::insert_runtime_arg(total_supply.clone(), "total_supply", &mut named_args);
                                        named_args
                                    },
                                )
                                .with_amount(self.attached_value),
                            )
                    }

                    /// Returns the total supply of the token.
                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_total_supply(&self) -> OdraResult<U256> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                odra::prelude::string::String::from("total_supply"),
                                false,
                                {
                                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                                    if self.attached_value > odra::casper_types::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }
                    /// Pay to mint.
                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_pay_to_mint(&mut self) -> OdraResult<()> {
                        self.env
                            .call_contract(
                                self.address,
                                odra::CallDef::new(
                                        odra::prelude::string::String::from("pay_to_mint"),
                                        true,
                                        {
                                            let mut named_args = odra::casper_types::RuntimeArgs::new();
                                            if self.attached_value > odra::casper_types::U512::zero() {
                                                let _ = named_args.insert("amount", self.attached_value);
                                            }
                                            named_args
                                        },
                                    )
                                    .with_amount(self.attached_value),
                            )
                    }
                    /// Approve.
                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_approve(
                        &mut self,
                        to: &Address,
                        amount: &U256,
                        msg: Maybe<String>,
                    ) -> OdraResult<()> {
                        self.env
                            .call_contract(
                                self.address,
                                odra::CallDef::new(
                                        odra::prelude::string::String::from("approve"),
                                        true,
                                        {
                                            let mut named_args = odra::casper_types::RuntimeArgs::new();
                                            if self.attached_value > odra::casper_types::U512::zero() {
                                                let _ = named_args.insert("amount", self.attached_value);
                                            }
                                            odra::args::EntrypointArgument::insert_runtime_arg(to.clone(), "to", &mut named_args);
                                            odra::args::EntrypointArgument::insert_runtime_arg(amount.clone(), "amount", &mut named_args);
                                            odra::args::EntrypointArgument::insert_runtime_arg(msg.clone(), "msg", &mut named_args);
                                            named_args
                                        },
                                    )
                                    .with_amount(self.attached_value),
                            )
                    }
                    /// Airdrops the given amount to the given addresses.
                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_airdrop(&self, to: &[Address], amount: &U256) -> OdraResult<()> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                odra::prelude::string::String::from("airdrop"),
                                false,
                                {
                                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                                    if self.attached_value > odra::casper_types::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    odra::args::EntrypointArgument::insert_runtime_arg(to.clone(), "to", &mut named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(amount.clone(), "amount", &mut named_args);
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }
                }

                impl odra::contract_def::HasIdent for Erc20HostRef {
                    fn ident() -> odra::prelude::string::String {
                        Erc20::ident()
                    }
                }

                #[allow(missing_docs)]
                /// [Erc20] contract constructor arguments.
                #[derive(odra::IntoRuntimeArgs)]
                pub struct Erc20InitArgs {
                    pub total_supply: Option<U256>,
                }

                impl odra::host::InitArgs for Erc20InitArgs {
                }

                impl odra::host::EntryPointsCallerProvider for Erc20HostRef {
                    fn entry_points_caller(env: &odra::host::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                        let entry_points = odra::prelude::vec![
                            odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("init"), odra::prelude::vec![
                                odra::entry_point_callback::Argument::new::<Option<U256> >(odra::prelude::string::String::from("total_supply"))
                            ]),
                            odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                            odra::entry_point_callback::EntryPoint::new_payable(odra::prelude::string::String::from("pay_to_mint"), odra::prelude::vec![]),
                            odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("approve"), odra::prelude::vec![
                                odra::entry_point_callback::Argument::new::<Address>(odra::prelude::string::String::from("to")),
                                odra::entry_point_callback::Argument::new::<U256>(odra::prelude::string::String::from("amount")),
                                odra::entry_point_callback::Argument::new::<Maybe<String> >(odra::prelude::string::String::from("msg"))
                            ]),
                            odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("airdrop"), odra::prelude::vec![
                                odra::entry_point_callback::Argument::new::<odra::prelude::vec::Vec<Address> >(odra::prelude::string::String::from("to")),
                                odra::entry_point_callback::Argument::new::<U256>(odra::prelude::string::String::from("amount"))
                            ])
                        ];
                        odra::entry_point_callback::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                            match call_def.entry_point() {
                                "init" => {
                                    let result = __erc20_exec_parts::execute_init(contract_env);
                                    odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                                }
                                "total_supply" => {
                                    let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                    odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                                }
                                "pay_to_mint" => {
                                    let result = __erc20_exec_parts::execute_pay_to_mint(contract_env);
                                    odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                                }
                                "approve" => {
                                    let result = __erc20_exec_parts::execute_approve(contract_env);
                                    odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                                }
                                "airdrop" => {
                                    let result = __erc20_exec_parts::execute_airdrop(contract_env);
                                    odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                                }
                                name => Err(OdraError::VmError(
                                    odra::VmError::NoSuchMethod(odra::prelude::String::from(name))
                                ))
                            }
                        })
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

                /// [Erc20] Host Ref.
                pub struct Erc20HostRef {
                    address: Address,
                    env: odra::host::HostEnv,
                    attached_value: odra::casper_types::U512
                }

                impl odra::host::HostRef for Erc20HostRef {
                    fn new(address: Address, env: odra::host::HostEnv) -> Self {
                        Self {
                            address,
                            env,
                            attached_value: Default::default()
                        }
                    }

                    fn with_tokens(&self, tokens: odra::casper_types::U512) -> Self {
                        Self {
                            address: self.address,
                            env: self.env.clone(),
                            attached_value: tokens
                        }
                    }

                    fn address(&self) -> &Address {
                        &self.address
                    }

                    fn env(&self) -> &odra::host::HostEnv {
                        &self.env
                    }

                    fn get_event<T>(&self, index: i32) -> Result<T, odra::EventError>
                    where
                        T: odra::casper_types::bytesrepr::FromBytes + odra::casper_event_standard::EventInstance,
                    {
                        self.env.get_event(self, index)
                    }

                    fn last_call(&self) -> odra::ContractCallResult {
                        self.env.last_call_result(self.address)
                    }
                }


                impl IErc20 for Erc20HostRef {
                    fn total_supply(&self) -> U256 {
                        self.try_total_supply().unwrap()
                    }

                    fn pay_to_mint(&mut self) {
                        self.try_pay_to_mint().unwrap()
                    }
                }

                impl Erc20HostRef {
                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_total_supply(&self) -> OdraResult<U256> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                odra::prelude::string::String::from("total_supply"),
                                false,
                                {
                                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                                    if self.attached_value > odra::casper_types::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }

                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_pay_to_mint(&mut self) -> OdraResult<()> {
                        self.env
                            .call_contract(
                                self.address,
                                odra::CallDef::new(
                                    odra::prelude::string::String::from("pay_to_mint"),
                                    true,
                                    {
                                        let mut named_args = odra::casper_types::RuntimeArgs::new();
                                        if self.attached_value > odra::casper_types::U512::zero() {
                                            let _ = named_args.insert("amount", self.attached_value);
                                        }
                                        named_args
                                    },
                                )
                                .with_amount(self.attached_value),
                            )
                    }
                }

                impl odra::contract_def::HasIdent for Erc20HostRef {
                    fn ident() -> odra::prelude::string::String {
                        Erc20::ident()
                    }
                }

                impl odra::host::EntryPointsCallerProvider for Erc20HostRef {
                    fn entry_points_caller(env: &odra::host::HostEnv) -> odra::entry_point_callback::EntryPointsCaller {
                        let entry_points = odra::prelude::vec![
                            odra::entry_point_callback::EntryPoint::new(odra::prelude::string::String::from("total_supply"), odra::prelude::vec![]),
                            odra::entry_point_callback::EntryPoint::new_payable(odra::prelude::string::String::from("pay_to_mint"), odra::prelude::vec![])
                        ];
                        odra::entry_point_callback::EntryPointsCaller::new(env.clone(), entry_points, |contract_env, call_def| {
                            match call_def.entry_point() {
                                "total_supply" => {
                                    let result = __erc20_exec_parts::execute_total_supply(contract_env);
                                    odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                                }
                                "pay_to_mint" => {
                                    let result = __erc20_exec_parts::execute_pay_to_mint(contract_env);
                                    odra::casper_types::bytesrepr::ToBytes::to_bytes(&result).map(Into::into).map_err(|err| OdraError::ExecutionError(err.into()))
                                }
                                name => Err(OdraError::VmError(
                                    odra::VmError::NoSuchMethod(odra::prelude::String::from(name)),
                                ))
                            }
                        })
                    }
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}
