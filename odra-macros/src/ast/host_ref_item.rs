use crate::{
    ir::ModuleImplIR,
    utils::{self, misc::AsBlock}
};
use derive_try_from_ref::TryFromRef;
use proc_macro2::Ident;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse_quote;

use super::{fn_utils::FnItem, ref_utils};

#[derive(syn_derive::ToTokens)]
struct HostRefStructItem {
    doc: syn::Attribute,
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

        let comment = format!(" [{}] Host Ref.", module.module_str()?);
        let doc = parse_quote!(#[doc = #comment]);

        Ok(Self {
            doc,
            vis: utils::syn::visibility_pub(),
            struct_token: Default::default(),
            ident: module.host_ref_ident()?,
            fields: named_fields.into()
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct HasIdentTraitImplItem {
    impl_token: syn::token::Impl,
    trait_ty: syn::Type,
    for_token: syn::token::For,
    ref_ident: Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    ident_fn: IdentFnItem
}

impl TryFrom<&'_ ModuleImplIR> for HasIdentTraitImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            trait_ty: utils::ty::has_ident(),
            for_token: Default::default(),
            ref_ident: module.host_ref_ident()?,
            brace_token: Default::default(),
            ident_fn: module.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct HostRefTraitImplItem {
    impl_token: syn::token::Impl,
    trait_ty: syn::Type,
    for_token: syn::token::For,
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
    last_call_fn: LastCallFnItem
}

impl TryFrom<&'_ ModuleImplIR> for HostRefTraitImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            trait_ty: utils::ty::host_ref(),
            for_token: Default::default(),
            ref_ident: module.host_ref_ident()?,
            brace_token: Default::default(),
            new_fn: NewFnItem,
            with_tokens_fn: WithTokensFnItem,
            address_fn: AddressFnItem,
            env_fn: EnvFnItem,
            get_event_fn: GetEventFnItem,
            last_call_fn: LastCallFnItem
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct HostRefImplItem {
    impl_token: syn::token::Impl,
    trait_name: Option<Ident>,
    for_token: Option<syn::token::For>,
    ref_ident: Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    functions: Vec<syn::ItemFn>
}

impl TryFrom<&'_ ModuleImplIR> for HostRefImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        // If module implements a trait, set trait name
        let trait_name = module.impl_trait_ident();
        let for_token: Option<syn::token::For> = match module.is_trait_impl() {  
            true => Some(Default::default()),
            false => None
        };
        
        Ok(Self {
            impl_token: Default::default(),
            trait_name,
            for_token,
            ref_ident: module.host_ref_ident()?,
            brace_token: Default::default(),
            functions: module
                .host_functions()?
                .iter()
                .flat_map(|f| {
                    vec![
                        ref_utils::host_function_item(f),
                    ]
                })
                .collect()
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct HostRefTryImplItem {
    impl_token: syn::token::Impl,
    ref_ident: Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    functions: Vec<syn::ItemFn>
}

impl TryFrom<&'_ ModuleImplIR> for HostRefTryImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        
        Ok(Self {
            impl_token: Default::default(),
            ref_ident: module.host_ref_ident()?,
            brace_token: Default::default(),
            functions: module
                .host_functions()?
                .iter()
                .flat_map(|f| {
                    vec![
                        ref_utils::host_try_function_item(f),
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
            fn new(#address: #ty_address, #env: #ty_env) -> #ty_self {
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
            fn with_tokens(&self, tokens: #ty_u512) -> Self {
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
        let m_address = utils::member::address();
        let ident = utils::ident::address();
        let ty_address_ref = utils::ty::address_ref();
        let ty_self_ref = utils::ty::self_ref();

        tokens.extend(quote!(
            fn #ident(#ty_self_ref) -> #ty_address_ref {
                &#m_address
            }
        ));
    }
}

struct EnvFnItem;

impl ToTokens for EnvFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let m_env = utils::member::env();
        let ident = utils::ident::env();
        let ret_ty = utils::ty::host_env();
        let ty_self_ref = utils::ty::self_ref();

        tokens.extend(quote!(
            fn #ident(#ty_self_ref) -> &#ret_ty {
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
            fn get_event<T>(&self, index: i32) -> Result<T, #ty_ev_error>
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
            fn last_call(&self) -> #ty_result {
                #m_env.last_call_result(#m_address)
            }
        ))
    }
}

#[derive(syn_derive::ToTokens)]
struct IdentFnItem {
    fn_item: FnItem
}

impl TryFrom<&'_ ModuleImplIR> for IdentFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let ident = utils::ident::ident();
        let ty_string = utils::ty::string();
        let module_ident = module.module_ident()?;
        let ret_ty: syn::ReturnType = utils::misc::ret_ty(&ty_string);
        let expr: syn::Expr = parse_quote!(#module_ident::#ident());

        Ok(Self {
            fn_item: FnItem::new(&ident, vec![], ret_ty, expr.as_block())
        })
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct HostRefItem {
    struct_item: HostRefStructItem,
    trait_impl_item: HostRefTraitImplItem,
    impl_item: HostRefImplItem,
    try_impl_item: HostRefTryImplItem
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
            /// [Erc20] Host Ref.
            pub struct Erc20HostRef {
                address: odra::Address,
                env: odra::host::HostEnv,
                attached_value: odra::casper_types::U512
            }

            impl odra::host::HostRef for Erc20HostRef {
                fn new(address: odra::Address, env: odra::host::HostEnv) -> Self {
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

                fn address(&self) -> &odra::Address {
                    &self.address
                }

                fn env(&self) -> &odra::host::HostEnv {
                    &self.env
                }

                fn get_event<T>(&self, index: i32) -> Result<T, odra::EventError>
                where
                    T: odra::casper_types::bytesrepr::FromBytes + odra::casper_event_standard::EventInstance,
                {
                    self.env.get_event(&self.address, index)
                }

                fn last_call(&self) -> odra::ContractCallResult {
                    self.env.last_call_result(self.address)
                }
            }

            impl Erc20HostRef {
                /// Returns the total supply of the token.
                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_total_supply(&self) -> odra::OdraResult<U256> {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            String::from("total_supply"),
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

                /// Returns the total supply of the token.
                pub fn total_supply(&self) -> U256 {
                    self.try_total_supply().unwrap()
                }

                /// Pay to mint.
                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_pay_to_mint(&mut self) -> odra::OdraResult<()> {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("pay_to_mint"),
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

                /// Pay to mint.
                pub fn pay_to_mint(&mut self) {
                    self.try_pay_to_mint().unwrap()
                }

                /// Approve.
                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_approve(
                    &mut self,
                    to: Address,
                    amount: U256,
                ) -> odra::OdraResult<()> {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("approve"),
                                    true,
                                    {
                                        let mut named_args = odra::casper_types::RuntimeArgs::new();
                                        if self.attached_value > odra::casper_types::U512::zero() {
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

                /// Approve.
                pub fn approve(&mut self, to: Address, amount: U256) {
                    self.try_approve(to, amount).unwrap()
                }

                /// Airdrops the given amount to the given addresses.
                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_airdrop(&self, to: odra::prelude::vec::Vec<Address>, amount: U256) -> odra::OdraResult<()> {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            String::from("airdrop"),
                            false,
                            {
                                let mut named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = named_args.insert("amount", self.attached_value);
                                }
                                let _ = named_args.insert("to", to);
                                let _ = named_args.insert("amount", amount);
                                named_args
                            }
                        ).with_amount(self.attached_value),
                    )
                }

                /// Airdrops the given amount to the given addresses.
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
            /// [Erc20] Host Ref.
            pub struct Erc20HostRef {
                address: odra::Address,
                env: odra::host::HostEnv,
                attached_value: odra::casper_types::U512
            }

            impl odra::host::HostRef for Erc20HostRef {
                fn new(address: odra::Address, env: odra::host::HostEnv) -> Self {
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

                fn address(&self) -> &odra::Address {
                    &self.address
                }

                fn env(&self) -> &odra::host::HostEnv {
                    &self.env
                }

                fn get_event<T>(&self, index: i32) -> Result<T, odra::EventError>
                where
                    T: odra::casper_types::bytesrepr::FromBytes + odra::casper_event_standard::EventInstance,
                {
                    self.env.get_event(&self.address, index)
                }

                fn last_call(&self) -> odra::ContractCallResult {
                    self.env.last_call_result(self.address)
                }
            }

            impl Erc20HostRef {
                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_total_supply(&self) -> odra::OdraResult<U256> {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            String::from("total_supply"),
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

                pub fn total_supply(&self) -> U256 {
                    self.try_total_supply().unwrap()
                }

                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_pay_to_mint(&mut self) -> odra::OdraResult<()>  {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("pay_to_mint"),
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
            /// [Erc20] Host Ref.
            pub struct Erc20HostRef {
                address: odra::Address,
                env: odra::host::HostEnv,
                attached_value: odra::casper_types::U512
            }

            impl odra::host::HostRef for Erc20HostRef {
                fn new(address: odra::Address, env: odra::host::HostEnv) -> Self {
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

                fn address(&self) -> &odra::Address {
                    &self.address
                }

                fn env(&self) -> &odra::host::HostEnv {
                    &self.env
                }

                fn get_event<T>(&self, index: i32) -> Result<T, odra::EventError>
                where
                T: odra::casper_types::bytesrepr::FromBytes + odra::casper_event_standard::EventInstance,
                {
                    self.env.get_event(&self.address, index)
                }

                fn last_call(&self) -> odra::ContractCallResult {
                    self.env.last_call_result(self.address)
                }
            }

            impl Erc20HostRef {
                /// Returns the total supply of the token.
                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_total_supply(&self) -> odra::OdraResult<U256> {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            String::from("total_supply"),
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

                /// Returns the total supply of the token.
                pub fn total_supply(&self) -> U256 {
                    self.try_total_supply().unwrap()
                }

                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_get_owner(&self) -> odra::OdraResult<Address> {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("get_owner"),
                                    false,
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

                pub fn get_owner(&self) -> Address {
                    self.try_get_owner().unwrap()
                }

                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_set_owner(&mut self, new_owner: Address) -> odra::OdraResult<()> {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("set_owner"),
                                    true,
                                    {
                                        let mut named_args = odra::casper_types::RuntimeArgs::new();
                                        if self.attached_value > odra::casper_types::U512::zero() {
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

                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_name(&self) -> odra::OdraResult<String> {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("name"),
                                    false,
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

                pub fn name(&self) -> String {
                    self.try_name().unwrap()
                }

                /// Does not fail in case of error, returns `odra::OdraResult` instead.
                pub fn try_symbol(&self) -> odra::OdraResult<String> {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                    String::from("symbol"),
                                    false,
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

                pub fn symbol(&self) -> String {
                    self.try_symbol().unwrap()
                 }
            }
        };
        let actual = HostRefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
