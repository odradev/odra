use crate::{ast::ref_utils, ir::ModuleIR, utils};
use derive_try_from::TryFromRef;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse_quote;

#[derive(syn_derive::ToTokens)]
struct ContractRefStructItem {
    vis: syn::Visibility,
    struct_token: syn::token::Struct,
    ident: syn::Ident,
    fields: syn::Fields
}

impl TryFrom<&'_ ModuleIR> for ContractRefStructItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let address = utils::ident::address();
        let env = utils::ident::env();
        let ty_address = utils::ty::address();
        let ty_contract_env = utils::ty::contract_env();
        let named_fields: syn::FieldsNamed = parse_quote!({
            #env: Rc<#ty_contract_env>,
            #address: #ty_address,
        });

        Ok(Self {
            vis: utils::syn::visibility_pub(),
            struct_token: Default::default(),
            ident: module.contract_ref_ident()?,
            fields: named_fields.into()
        })
    }
}

struct AddressFnItem;

impl ToTokens for AddressFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty_address = utils::ty::address();
        let m_address = utils::member::address();

        tokens.extend(quote!(
            pub fn address(&self) -> &#ty_address {
                &#m_address
            }
        ))
    }
}

#[derive(syn_derive::ToTokens)]
struct ContractRefImplItem {
    impl_token: syn::token::Impl,
    ref_ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    address_fn: AddressFnItem,
    #[syn(in = brace_token)]
    #[to_tokens(|tokens, val| tokens.append_all(val))]
    functions: Vec<syn::ItemFn>
}

impl TryFrom<&'_ ModuleIR> for ContractRefImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            ref_ident: module.contract_ref_ident()?,
            brace_token: Default::default(),
            address_fn: AddressFnItem,
            functions: module
                .functions()
                .iter()
                .map(ref_utils::contract_function_item)
                .collect()
        })
    }
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleIR)]
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
        let module = test_utils::mock_module();
        let expected = quote! {
            pub struct Erc20ContractRef {
                env: Rc<odra::ContractEnv>,
                address: odra::Address,
            }

            impl Erc20ContractRef {
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
}
