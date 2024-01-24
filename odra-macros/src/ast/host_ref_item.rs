use crate::{ir::ModuleImplIR, utils};
use derive_try_from_ref::TryFromRef;
use proc_macro2::Ident;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse_quote;

use super::ref_utils;

#[derive(syn_derive::ToTokens)]
struct HostRefStructItem {
    vis: syn::Visibility,
    struct_token: syn::token::Struct,
    ident: syn::Ident,
    fields: syn::Fields
}

impl TryFrom<&'_ ModuleImplIR> for HostRefStructItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let address = utils::ident::address();
        let env = utils::ident::env();
        let attached_value = utils::ident::attached_value();

        let ty_address = utils::ty::address();
        let ty_host_env = utils::ty::host_env();
        let ty_u512 = utils::ty::u512();

        let named_fields: syn::FieldsNamed = parse_quote!({
            #address: #ty_address,
            #env: #ty_host_env,
            #attached_value: #ty_u512
        });
        Ok(Self {
            vis: utils::syn::visibility_pub(),
            struct_token: Default::default(),
            ident: module.host_ref_ident()?,
            fields: named_fields.into()
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct HostRefImplItem {
    impl_token: syn::token::Impl,
    ref_ident: Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    new_fn: NewFnItem,
    #[syn(in = brace_token)]
    with_tokens_fn: WithTokensFnItem,
    #[syn(in = brace_token)]
    address_fn: AddressFnItem,
    #[syn(in = brace_token)]
    env_fn: EnvFnItem,
    #[syn(in = brace_token)]
    get_event_fn: GetEventFnItem,
    #[syn(in = brace_token)]
    last_call_fn: LastCallFnItem,
    #[syn(in = brace_token)]
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    functions: Vec<syn::ItemFn>
}

impl TryFrom<&'_ ModuleImplIR> for HostRefImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            ref_ident: module.host_ref_ident()?,
            brace_token: Default::default(),
            new_fn: NewFnItem,
            with_tokens_fn: WithTokensFnItem,
            address_fn: AddressFnItem,
            env_fn: EnvFnItem,
            get_event_fn: GetEventFnItem,
            last_call_fn: LastCallFnItem,
            functions: module
                .host_functions()?
                .iter()
                .flat_map(|f| {
                    vec![
                        ref_utils::host_try_function_item(f),
                        ref_utils::host_function_item(f),
                    ]
                })
                .collect()
        })
    }
}

struct NewFnItem;

impl ToTokens for NewFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty_address = utils::ty::address();
        let ty_env = utils::ty::host_env();
        let env = utils::ident::env();
        let address = utils::ident::address();
        let attached_value = utils::ident::attached_value();
        let default = utils::expr::default();
        let ty_self = utils::ty::_Self();

        tokens.extend(quote!(
            pub fn new(#address: #ty_address, #env: #ty_env) -> #ty_self {
                #ty_self {
                    #address,
                    #env,
                    #attached_value: #default
                }
            }
        ));
    }
}

struct WithTokensFnItem;

impl ToTokens for WithTokensFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let m_address = utils::member::address();
        let m_env = utils::member::env();

        let ty_u512 = utils::ty::u512();

        let address = utils::ident::address();
        let attached_value = utils::ident::attached_value();
        let env = utils::ident::env();

        tokens.extend(quote!(
            pub fn with_tokens(&self, tokens: #ty_u512) -> Self {
                Self {
                    #address: #m_address,
                    #env: #m_env.clone(),
                    #attached_value: tokens
                }
            }
        ));
    }
}

struct AddressFnItem;

impl ToTokens for AddressFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vis = utils::syn::visibility_pub();
        let m_address = utils::member::address();
        let ident = utils::ident::address();
        let ty_address_ref = utils::ty::address_ref();
        let ty_self_ref = utils::ty::self_ref();

        tokens.extend(quote!(
            #vis fn #ident(#ty_self_ref) -> #ty_address_ref {
                &#m_address
            }
        ));
    }
}

struct EnvFnItem;

impl ToTokens for EnvFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let vis = utils::syn::visibility_pub();
        let m_env = utils::member::env();
        let ident = utils::ident::env();
        let ret_ty = utils::ty::host_env();
        let ty_self_ref = utils::ty::self_ref();

        tokens.extend(quote!(
            #vis fn #ident(#ty_self_ref) -> &#ret_ty {
                &#m_env
            }
        ));
    }
}

struct GetEventFnItem;

impl ToTokens for GetEventFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let m_env = utils::member::env();
        let m_address = utils::member::address();

        let ty_ev = utils::ty::event_instance();
        let ty_from_bytes = utils::ty::from_bytes();
        let ty_ev_error = utils::ty::event_error();

        tokens.extend(quote!(
            pub fn get_event<T>(&self, index: i32) -> Result<T, #ty_ev_error>
            where
                T: #ty_from_bytes + #ty_ev
            {
                #m_env.get_event(&#m_address, index)
            }
        ));
    }
}

struct LastCallFnItem;

impl ToTokens for LastCallFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let m_env = utils::member::env();
        let m_address = utils::member::address();
        let ty_result = utils::ty::contract_call_result();
        tokens.extend(quote!(
            pub fn last_call(&self) -> #ty_result {
                #m_env.last_call().contract_last_call(#m_address)
            }
        ))
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct HostRefItem {
    struct_item: HostRefStructItem,
    impl_item: HostRefImplItem
}

#[cfg(test)]
mod ref_item_tests {
    use super::HostRefItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn host_ref() {
        let module = test_utils::mock::module_impl();
        let expected = quote! {
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
                    self.env
                        .call_contract(
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
                                    },
                                )
                                .with_amount(self.attached_value),
                        )
                }

                pub fn pay_to_mint(&mut self) {
                    self.try_pay_to_mint().unwrap()
                }

                pub fn try_approve(
                    &mut self,
                    to: Address,
                    amount: U256,
                ) -> Result<(), odra::OdraError> {
                    self.env
                        .call_contract(
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
                                    },
                                )
                                .with_amount(self.attached_value),
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
        };
        let actual = HostRefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn host_trait_impl_ref() {
        let module = test_utils::mock::module_trait_impl();
        let expected = quote! {
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
                    self.env
                        .call_contract(
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
                                    },
                                )
                                .with_amount(self.attached_value),
                        )
                }

                pub fn pay_to_mint(&mut self) {
                    self.try_pay_to_mint().unwrap()
                }
            }
        };
        let actual = HostRefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn host_ref_delegation() {
        let module = test_utils::mock::module_delegation();
        let expected = quote! {
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

                pub fn try_get_owner(&self) -> Result<Address, odra::OdraError> {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("get_owner"),
                                    false,
                                    {
                                        let mut named_args = odra::RuntimeArgs::new();
                                        if self.attached_value > odra::U512::zero() {
                                            let _ = named_args.insert("amount", self.attached_value);
                                        }
                                        named_args
                                    },
                                )
                                .with_amount(self.attached_value),
                        )
                }

                pub fn get_owner(&self) -> Address {
                    self.try_get_owner().unwrap()
                }

                pub fn try_set_owner(&mut self, new_owner: Address) -> Result<(), odra::OdraError> {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("set_owner"),
                                    true,
                                    {
                                        let mut named_args = odra::RuntimeArgs::new();
                                        if self.attached_value > odra::U512::zero() {
                                            let _ = named_args.insert("amount", self.attached_value);
                                        }
                                        let _ = named_args.insert("new_owner", new_owner);
                                        named_args
                                    },
                                )
                                .with_amount(self.attached_value),
                        )
                }

                pub fn set_owner(&mut self, new_owner: Address) {
                    self.try_set_owner(new_owner).unwrap()
                }

                pub fn try_name(&self) -> Result<String, odra::OdraError> {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("name"),
                                    false,
                                    {
                                        let mut named_args = odra::RuntimeArgs::new();
                                        if self.attached_value > odra::U512::zero() {
                                            let _ = named_args.insert("amount", self.attached_value);
                                        }
                                        named_args
                                    },
                                )
                                .with_amount(self.attached_value),
                        )
                }

                pub fn name(&self) -> String {
                    self.try_name().unwrap()
                }

                pub fn try_symbol(&self) -> Result<String, odra::OdraError> {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("symbol"),
                                    false,
                                    {
                                        let mut named_args = odra::RuntimeArgs::new();
                                        if self.attached_value > odra::U512::zero() {
                                            let _ = named_args.insert("amount", self.attached_value);
                                        }
                                        named_args
                                    },
                                )
                                .with_amount(self.attached_value),
                        )
                }

                pub fn symbol(&self) -> String {
                    self.try_symbol().unwrap()
                 }
            }
        };
        let actual = HostRefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
