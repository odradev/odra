use crate::{ast::ref_utils, ir::ModuleImplIR, utils};
use derive_try_from_ref::TryFromRef;
use quote::{ToTokens, TokenStreamExt};

use super::ref_utils::{SchemaErrorsItem, SchemaEventsItem};

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct RefItem {
    struct_item: ContractRefStructItem,
    trait_impl_item: ContractRefTraitImplItem,
    impl_item: ContractRefImplItem,
    schema_errors_item: SchemaErrorsItem,
    schema_events_item: SchemaEventsItem
}

pub(super) struct ContractRefStructItem {
    module_ident: syn::Ident,
    ident: syn::Ident
}

impl ToTokens for ContractRefStructItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let ty_rc_contract_env = utils::ty::rc_contract_env();
        let ty_address = utils::ty::address();
        let ty_u512 = utils::ty::u512();
        let comment = format!(" [{}] Contract Ref.", self.module_ident);
        tokens.append_all(quote::quote! {
            #[doc = #comment]
            pub struct #ident {
                env: #ty_rc_contract_env,
                address: #ty_address,
                attached_value: #ty_u512,
            }
        })
    }
}

impl TryFrom<&'_ ModuleImplIR> for ContractRefStructItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: module.module_ident()?,
            ident: module.contract_ref_ident()?
        })
    }
}

pub(super) struct ContractRefTraitImplItem {
    ref_ident: syn::Ident,
    new_fn: NewFnItem,
    address_fn: AddressFnItem,
    with_tokens_fn: WithTokensFnItem
}

impl TryFrom<&'_ ModuleImplIR> for ContractRefTraitImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            ref_ident: module.contract_ref_ident()?,
            new_fn: NewFnItem,
            address_fn: AddressFnItem,
            with_tokens_fn: WithTokensFnItem
        })
    }
}

impl ToTokens for ContractRefTraitImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = utils::ty::contract_ref();
        let ident = &self.ref_ident;
        let fn_new = &self.new_fn;
        let fn_address = &self.address_fn;
        let fn_with_tokens = &self.with_tokens_fn;
        tokens.append_all(quote::quote! {
            impl #ty for #ident {
                #fn_new
                #fn_address
                #fn_with_tokens
            }
        })
    }
}

struct ContractRefImplItem {
    trait_name: Option<syn::Ident>,
    for_token: Option<syn::token::For>,
    ref_ident: syn::Ident,
    functions: Vec<syn::ItemFn>
}

impl TryFrom<&'_ ModuleImplIR> for ContractRefImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        // If module implements a trait, set trait name
        let trait_name = module.impl_trait_ident();

        let for_token: Option<syn::token::For> = module.is_trait_impl().then(Default::default);
        Ok(Self {
            trait_name,
            for_token,
            ref_ident: module.contract_ref_ident()?,
            functions: module
                .functions()?
                .iter()
                .map(|fun| ref_utils::contract_function_item(fun, module.is_trait_impl()))
                .collect()
        })
    }
}

impl ToTokens for ContractRefImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let trait_name = &self.trait_name;
        let for_token = &self.for_token;
        let ref_ident = &self.ref_ident;
        let functions = &self.functions;

        tokens.append_all(quote::quote! {
            impl #trait_name #for_token #ref_ident {
                #(#functions)*
            }
        })
    }
}

struct AddressFnItem;

impl ToTokens for AddressFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(quote::quote! {
            fn address(&self) -> &Address {
                &self.address
            }
        })
    }
}

struct NewFnItem;

impl ToTokens for NewFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty_address = utils::ty::address();
        let ty_rc_contract_env = utils::ty::rc_contract_env();
        let ty_u512 = utils::ty::u512();
        tokens.append_all(quote::quote! {
            fn new(env: #ty_rc_contract_env, address: #ty_address) -> Self {
                Self {
                    env,
                    address,
                    attached_value: #ty_u512::zero()
                }
            }
        })
    }
}

pub(crate) struct WithTokensFnItem;

impl ToTokens for WithTokensFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let m_address = utils::member::address();
        let m_env = utils::member::env();

        let ty_u512 = utils::ty::u512();

        let address = utils::ident::address();
        let attached_value = utils::ident::attached_value();
        let env = utils::ident::env();

        tokens.extend(quote::quote!(
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

#[cfg(test)]
mod test {
    use super::RefItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn contract_ref() {
        let module = test_utils::mock::module_impl();
        let expected = quote! {
            /// [Erc20] Contract Ref.
            pub struct Erc20ContractRef {
                env: Rc<odra::ContractEnv>,
                address: Address,
                attached_value: odra::casper_types::U512,
            }

            impl odra::ContractRef for Erc20ContractRef {
                fn new(env: Rc<odra::ContractEnv>, address: Address) -> Self {
                    Self {
                        env,
                        address,
                        attached_value: odra::casper_types::U512::zero()
                    }
                }

                fn address(&self) -> &Address {
                    &self.address
                }

                fn with_tokens(&self, tokens: odra::casper_types::U512) -> Self {
                    Self {
                        address: self.address,
                        env: self.env.clone(),
                        attached_value: tokens,
                    }
                }
            }

            impl Erc20ContractRef {
                /// Initializes the contract with the given parameters.
                pub fn init(&mut self, total_supply: Option<U256>) {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("init"),
                            true,
                            {
                                let mut named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = named_args.insert("amount", self.attached_value);
                                }
                                odra::args::EntrypointArgument::insert_runtime_arg(total_supply, "total_supply", &mut named_args);
                                named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
                }

                /// Upgrades the contract with the given parameters.
                pub fn upgrade(&mut self, total_supply: Option<U256>) {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("upgrade"),
                            true,
                            {
                                let mut named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = named_args.insert("amount", self.attached_value);
                                }
                                odra::args::EntrypointArgument::insert_runtime_arg(total_supply, "total_supply", &mut named_args);
                                named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
                }

                /// Returns the total supply of the token.
                pub fn total_supply(&self) -> U256 {
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
                        )
                        .with_amount(self.attached_value),
                    )
                }

                /// Pay to mint.
                pub fn pay_to_mint(&mut self) {
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
                                }
                            )
                            .with_amount(self.attached_value),
                        )
                }

                /// Approve.
                pub fn approve(&mut self, to: &Address, amount: &U256, msg: Maybe<String>) {
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
                                    odra::args::EntrypointArgument::insert_runtime_arg(to, "to", &mut named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(amount, "amount", &mut named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(msg, "msg", &mut named_args);
                                    named_args
                                }
                            )
                            .with_amount(self.attached_value),
                        )
                }

                /// Airdrops the given amount to the given addresses.
                pub fn airdrop(&self, to: &[Address], amount: &U256) {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                odra::prelude::string::String::from("airdrop"),
                                false,
                                {
                                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                                    if self.attached_value > odra::casper_types::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    odra::args::EntrypointArgument::insert_runtime_arg(to, "to", &mut named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(amount, "amount", &mut named_args);
                                    named_args
                                }
                            )
                            .with_amount(self.attached_value),
                        )
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for Erc20ContractRef {}

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for Erc20ContractRef {}
        };
        let actual = RefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn contract_trait_impl_ref() {
        let module = test_utils::mock::module_trait_impl();
        let expected = quote! {
            /// [Erc20] Contract Ref.
            pub struct Erc20ContractRef {
                env: Rc<odra::ContractEnv>,
                address: Address,
                attached_value: odra::casper_types::U512,
            }

            impl odra::ContractRef for Erc20ContractRef {
                fn new(env: Rc<odra::ContractEnv>, address: Address) -> Self {
                    Self {
                        env,
                        address,
                        attached_value: odra::casper_types::U512::zero()
                    }
                }

                fn address(&self) -> &Address {
                    &self.address
                }

                fn with_tokens(&self, tokens: odra::casper_types::U512) -> Self {
                    Self {
                        address: self.address,
                        env: self.env.clone(),
                        attached_value: tokens,
                    }
                }
            }

            impl IErc20 for Erc20ContractRef {
                fn total_supply(&self) -> U256 {
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
                        )
                        .with_amount(self.attached_value),
                    )
                }

                fn pay_to_mint(&mut self) {
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
                                }
                            )
                            .with_amount(self.attached_value),
                        )
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for Erc20ContractRef {}

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for Erc20ContractRef {}
        };
        let actual = RefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn contract_ref_delegate() {
        let module = test_utils::mock::module_delegation();
        let expected = quote! {
            /// [Erc20] Contract Ref.
            pub struct Erc20ContractRef {
                env: Rc<odra::ContractEnv>,
                address: Address,
                attached_value: odra::casper_types::U512,
            }

            impl odra::ContractRef for Erc20ContractRef {
                fn new(env: Rc<odra::ContractEnv>, address: Address) -> Self {
                    Self {
                        env,
                        address,
                        attached_value: odra::casper_types::U512::zero()
                    }
                }

                fn address(&self) -> &Address {
                    &self.address
                }

                fn with_tokens(&self, tokens: odra::casper_types::U512) -> Self {
                    Self {
                        address: self.address,
                        env: self.env.clone(),
                        attached_value: tokens,
                    }
                }
            }

            impl Erc20ContractRef {
                /// Returns the total supply of the token.
                pub fn total_supply(&self) -> U256 {
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
                        )
                        .with_amount(self.attached_value),
                    )
                }

                /// Returns the owner of the contract.
                pub fn get_owner(&self) -> Address {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                odra::prelude::string::String::from("get_owner"),
                                false,
                                {
                                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                                    if self.attached_value > odra::casper_types::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    named_args
                                }
                            )
                            .with_amount(self.attached_value),
                        )
                }

                /// Sets the owner of the contract.
                pub fn set_owner(&mut self, new_owner: Address) {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                odra::prelude::string::String::from("set_owner"),
                                true,
                                {
                                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                                    if self.attached_value > odra::casper_types::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    odra::args::EntrypointArgument::insert_runtime_arg(new_owner, "new_owner", &mut named_args);
                                    named_args
                                }
                            )
                            .with_amount(self.attached_value),
                        )
                }

                /// Returns the name of the token.
                pub fn name(&self) -> String {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                odra::prelude::string::String::from("name"),
                                false,
                                {
                                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                                    if self.attached_value > odra::casper_types::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    named_args
                                }
                            )
                            .with_amount(self.attached_value),
                        )
                }

                /// Delegated. See `self.metadata.symbol()` for details.
                pub fn symbol(&self) -> String {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                odra::prelude::string::String::from("symbol"),
                                false,
                                {
                                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                                    if self.attached_value > odra::casper_types::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    named_args
                                }
                            )
                            .with_amount(self.attached_value),
                        )
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for Erc20ContractRef {}

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for Erc20ContractRef {}
        };
        let actual = RefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn contract_ref_invalid_delegate() {
        let module = test_utils::mock::module_invalid_delegation();
        let actual = RefItem::try_from(&module);
        assert!(actual.is_err());
    }
}
