use crate::ast::fn_utils::{FnItem, SelfFnItem};
use crate::utils::misc::AsBlock;
use crate::{ast::ref_utils, ir::ModuleImplIR, utils};
use derive_try_from::TryFromRef;
use quote::TokenStreamExt;
use syn::parse_quote;

#[derive(syn_derive::ToTokens)]
struct ContractRefStructItem {
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
        let ty_contract_env = utils::ty::contract_env();
        let named_fields: syn::FieldsNamed = parse_quote!({
            pub #env: Rc<#ty_contract_env>,
            pub #address: #ty_address,
        });

        Ok(Self {
            vis: utils::syn::visibility_pub(),
            struct_token: Default::default(),
            ident: module.contract_ref_ident()?,
            fields: named_fields.into()
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct AddressFnItem {
    vis: syn::Visibility,
    fn_item: SelfFnItem
}

impl AddressFnItem {
    pub fn new() -> Self {
        let m_address = utils::member::address();
        let ret_expr: syn::Expr = parse_quote!(&#m_address);
        Self {
            vis: utils::syn::visibility_pub(),
            fn_item: SelfFnItem::new(
                &utils::ident::address(),
                utils::misc::ret_ty(&utils::ty::address_ref()),
                ret_expr.as_block()
            )
        }
    }
}

#[derive(syn_derive::ToTokens)]
struct NewFnItem {
    vis: syn::Visibility,
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
        Self {
            vis: utils::syn::visibility_pub(),
            fn_item: FnItem::new(&utils::ident::new(), args, ret_ty, ret_expr.as_block())
        }
    }
}

#[derive(syn_derive::ToTokens)]
struct ContractRefImplItem {
    impl_token: syn::token::Impl,
    ref_ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    new_fn: NewFnItem,
    #[syn(in = brace_token)]
    address_fn: AddressFnItem,
    #[syn(in = brace_token)]
    #[to_tokens(|tokens, val| tokens.append_all(val))]
    functions: Vec<syn::ItemFn>
}

impl TryFrom<&'_ ModuleImplIR> for ContractRefImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            ref_ident: module.contract_ref_ident()?,
            brace_token: Default::default(),
            new_fn: NewFnItem::new(),
            address_fn: AddressFnItem::new(),
            functions: module
                .functions()
                .iter()
                .map(ref_utils::contract_function_item)
                .collect()
        })
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
pub struct RefItem {
    struct_item: ContractRefStructItem,
    impl_item: ContractRefImplItem
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
            pub struct Erc20ContractRef {
                pub env: Rc<odra::ContractEnv>,
                pub address: odra::Address,
            }

            impl Erc20ContractRef {
                pub fn new(env: Rc<odra::ContractEnv>, address: odra::Address) -> Self {
                    Self { env, address }
                }

                // TODO: this means "address", can't be entrypoint name.
                pub fn address(&self) -> &odra::Address {
                    &self.address
                }

                pub fn init(&mut self, total_supply: Option<U256>) {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            String::from("init"),
                            {
                                let mut named_args = odra::RuntimeArgs::new();
                                let _ = named_args.insert("total_supply", total_supply);
                                named_args
                            }
                        ),
                    )
                }

                pub fn total_supply(&self) -> U256 {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            String::from("total_supply"),
                            {
                                let mut named_args = odra::RuntimeArgs::new();
                                named_args
                            }
                        ),
                    )
                }

                pub fn pay_to_mint(&mut self) {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                String::from("pay_to_mint"),
                                {
                                    let mut named_args = odra::RuntimeArgs::new();
                                    named_args
                                },
                            ),
                        )
                }

                pub fn approve(&mut self, to: Address, amount: U256) {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                String::from("approve"),
                                {
                                    let mut named_args = odra::RuntimeArgs::new();
                                    let _ = named_args.insert("to", to);
                                    let _ = named_args.insert("amount", amount);
                                    named_args
                                },
                            ),
                        )
                }
            }
        };
        let actual = RefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn contract_trait_impl_ref() {
        let module = test_utils::mock::module_trait_impl();
        let expected = quote! {
            pub struct Erc20ContractRef {
                pub env: Rc<odra::ContractEnv>,
                pub address: odra::Address,
            }

            impl Erc20ContractRef {
                pub fn new(env: Rc<odra::ContractEnv>, address: odra::Address) -> Self {
                    Self { env, address }
                }

                // TODO: this means "address", can't be entrypoint name.
                pub fn address(&self) -> &odra::Address {
                    &self.address
                }

                pub fn total_supply(&self) -> U256 {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            String::from("total_supply"),
                            {
                                let mut named_args = odra::RuntimeArgs::new();
                                named_args
                            }
                        ),
                    )
                }

                pub fn pay_to_mint(&mut self) {
                    self.env
                        .call_contract(
                            self.address,
                            odra::CallDef::new(
                                String::from("pay_to_mint"),
                                {
                                    let mut named_args = odra::RuntimeArgs::new();
                                    named_args
                                },
                            ),
                        )
                }
            }
        };
        let actual = RefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
