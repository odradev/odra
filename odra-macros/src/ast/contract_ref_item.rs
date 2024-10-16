use crate::ast::fn_utils::FnItem;
use crate::utils::misc::AsBlock;
use crate::utils::ty::contract_ref;
use crate::{ast::ref_utils, ir::ModuleImplIR, utils};
use derive_try_from_ref::TryFromRef;
use quote::TokenStreamExt;
use syn::parse_quote;

use super::ref_utils::{SchemaErrorsItem, SchemaEventsItem};

#[derive(syn_derive::ToTokens)]
struct ContractRefStructItem {
    doc: syn::Attribute,
    vis: syn::Visibility,
    struct_token: syn::token::Struct,
    ident: syn::Ident,
    fields: syn::Fields
}

impl TryFrom<&'_ ModuleImplIR> for ContractRefStructItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let address = utils::ident::address();
        let env = utils::ident::env();
        let ty_address = utils::ty::address();
        let ty_rc_contract_env = utils::ty::rc_contract_env();

        let named_fields: syn::FieldsNamed = parse_quote!({
            #env: #ty_rc_contract_env,
            #address: #ty_address,
        });

        let comment = format!(" [{}] Contract Ref.", module.module_str()?);
        let doc_attr = parse_quote!(#[doc = #comment]);

        Ok(Self {
            doc: doc_attr,
            vis: utils::syn::visibility_pub(),
            struct_token: Default::default(),
            ident: module.contract_ref_ident()?,
            fields: named_fields.into()
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct AddressFnItem {
    fn_item: FnItem
}

impl AddressFnItem {
    pub fn new() -> Self {
        let m_address = utils::member::address();
        let ret_expr: syn::Expr = parse_quote!(&#m_address);
        let fn_item = FnItem::new(
            &utils::ident::address(),
            vec![],
            utils::misc::ret_ty(&utils::ty::address_ref()),
            ret_expr.as_block()
        )
        .instanced();

        Self { fn_item }
    }
}

#[derive(syn_derive::ToTokens)]
struct NewFnItem {
    fn_item: FnItem
}

impl NewFnItem {
    fn new() -> Self {
        let ty_address = utils::ty::address();
        let m_env = utils::ident::env();
        let m_address = utils::ident::address();
        let ret_ty = utils::misc::ret_ty(&utils::ty::_Self());
        let ty_rc_contract_env = utils::ty::rc_contract_env();
        let args = vec![
            parse_quote!(#m_env: #ty_rc_contract_env),
            parse_quote!(#m_address: #ty_address),
        ];
        let ret_expr: syn::Expr = parse_quote!(Self { #m_env, #m_address });
        let fn_item = FnItem::new(&utils::ident::new(), args, ret_ty, ret_expr.as_block());
        Self { fn_item }
    }
}

#[derive(syn_derive::ToTokens)]
struct ContractRefTraitImplItem {
    impl_token: syn::token::Impl,
    trait_name: syn::Type,
    for_token: syn::token::For,
    ref_ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    new_fn: NewFnItem,
    #[syn(in = brace_token)]
    address_fn: AddressFnItem
}

impl TryFrom<&'_ ModuleImplIR> for ContractRefTraitImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            trait_name: contract_ref(),
            for_token: Default::default(),
            ref_ident: module.contract_ref_ident()?,
            brace_token: Default::default(),
            new_fn: NewFnItem::new(),
            address_fn: AddressFnItem::new()
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct ContractRefImplItem {
    impl_token: syn::token::Impl,
    trait_name: Option<syn::Ident>,
    for_token: Option<syn::token::For>,
    ref_ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    #[to_tokens(|tokens, val| tokens.append_all(val))]
    functions: Vec<syn::ItemFn>
}

impl TryFrom<&'_ ModuleImplIR> for ContractRefImplItem {
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
            ref_ident: module.contract_ref_ident()?,
            brace_token: Default::default(),
            functions: module
                .functions()?
                .iter()
                .map(|fun| ref_utils::contract_function_item(fun, module.is_trait_impl()))
                .collect()
        })
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
pub struct RefItem {
    struct_item: ContractRefStructItem,
    trait_impl_item: ContractRefTraitImplItem,
    impl_item: ContractRefImplItem,
    schema_errors_item: SchemaErrorsItem,
    schema_events_item: SchemaEventsItem,
}

#[cfg(test)]
mod ref_item_tests {
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
            }

            impl odra::ContractRef for Erc20ContractRef {
                fn new(env: Rc<odra::ContractEnv>, address: Address) -> Self {
                    Self { env, address }
                }

                fn address(&self) -> &Address {
                    &self.address
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
                                odra::args::EntrypointArgument::insert_runtime_arg(total_supply.clone(), "total_supply", &mut named_args);
                                named_args
                            }
                        ),
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
                                named_args
                            }
                        ),
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
                                    named_args
                                },
                            ),
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
                                    odra::args::EntrypointArgument::insert_runtime_arg(to.clone(), "to", &mut named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(amount.clone(), "amount", &mut named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(msg.clone(), "msg", &mut named_args);
                                    named_args
                                },
                            ),
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
                                    odra::args::EntrypointArgument::insert_runtime_arg(to.clone(), "to", &mut named_args);
                                    odra::args::EntrypointArgument::insert_runtime_arg(amount.clone(), "amount", &mut named_args);
                                    named_args
                                },
                            ),
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
            }

            impl odra::ContractRef for Erc20ContractRef {
                fn new(env: Rc<odra::ContractEnv>, address: Address) -> Self {
                    Self { env, address }
                }

                fn address(&self) -> &Address {
                    &self.address
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
                                named_args
                            }
                        ),
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
                                    named_args
                                },
                            ),
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
            }

            impl odra::ContractRef for Erc20ContractRef {
                fn new(env: Rc<odra::ContractEnv>, address: Address) -> Self {
                    Self { env, address }
                }

                fn address(&self) -> &Address {
                    &self.address
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
                                named_args
                            }
                        ),
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
                                    named_args
                                },
                            ),
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
                                    odra::args::EntrypointArgument::insert_runtime_arg(new_owner.clone(), "new_owner", &mut named_args);
                                    named_args
                                },
                            ),
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
                                    named_args
                                },
                            ),
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
                                    named_args
                                },
                            ),
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
