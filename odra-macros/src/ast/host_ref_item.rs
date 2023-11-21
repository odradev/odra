use crate::{ir::ModuleIR, utils};
use proc_macro2::Ident;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse_quote;

use super::ref_utils;

const CONSTRUCTOR_NAME: &str = "init";

#[derive(syn_derive::ToTokens)]
struct HostRefStructItem {
    vis: syn::Visibility,
    struct_token: syn::token::Struct,
    ident: syn::Ident,
    fields: syn::Fields
}

impl TryFrom<&'_ ModuleIR> for HostRefStructItem {
    type Error = syn::Error;

    fn try_from(value: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let named_fields: syn::FieldsNamed = parse_quote!({
            pub address: odra2::types::Address,
            pub env: odra2::HostEnv,
            pub attached_value: odra2::types::U512
        });
        Ok(Self {
            vis: utils::syn::visibility_pub(),
            struct_token: Default::default(),
            ident: value.host_ref_ident()?,
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
    with_tokens_fn: WithTokensFnItem,
    #[syn(in = brace_token)]
    get_event_fn: GetEventFnItem,
    #[syn(in = brace_token)]
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    functions: Vec<syn::ItemFn>
}

impl TryFrom<&'_ ModuleIR> for HostRefImplItem {
    type Error = syn::Error;

    fn try_from(value: &'_ ModuleIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            ref_ident: value.host_ref_ident()?,
            brace_token: Default::default(),
            with_tokens_fn: WithTokensFnItem,
            get_event_fn: GetEventFnItem,
            functions: value
                .functions()
                .iter()
                .filter(|f| f.name_str() != CONSTRUCTOR_NAME)
                .map(|f| {
                    vec![
                        ref_utils::host_try_function_item(f),
                        ref_utils::host_function_item(f),
                    ]
                })
                .flatten()
                .collect()
        })
    }
}

struct WithTokensFnItem;

impl ToTokens for WithTokensFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(
            pub fn with_tokens(&self, tokens: odra2::types::U512) -> Self {
                Self {
                    address: self.address,
                    env: self.env.clone(),
                    attached_value: tokens
                }
            }
        ));
    }
}

struct GetEventFnItem;

impl ToTokens for GetEventFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(
            pub fn get_event<T>(&self, index: i32) -> Result<T, odra2::event::EventError>
            where
                T: odra2::types::FromBytes + odra2::casper_event_standard::EventInstance
            {
                self.env.get_event(&self.address, index)
            }
        ));
    }
}

#[derive(syn_derive::ToTokens)]
pub struct HostRefItem {
    struct_item: HostRefStructItem,
    impl_item: HostRefImplItem
}

impl<'a> TryFrom<&'a ModuleIR> for HostRefItem {
    type Error = syn::Error;

    fn try_from(value: &'a ModuleIR) -> Result<Self, Self::Error> {
        Ok(Self {
            struct_item: value.try_into()?,
            impl_item: value.try_into()?
        })
    }
}

#[cfg(test)]
mod ref_item_tests {
    use super::HostRefItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn host_ref() {
        let module = test_utils::mock_module();
        let expected = quote! {
            pub struct Erc20HostRef {
                pub address: odra2::types::Address,
                pub env: odra2::HostEnv,
                pub attached_value: odra2::types::U512
            }

            impl Erc20HostRef {
                pub fn with_tokens(&self, tokens: odra2::types::U512) -> Self {
                    Self {
                        address: self.address,
                        env: self.env.clone(),
                        attached_value: tokens
                    }
                }

                pub fn get_event<T>(&self, index: i32) -> Result<T, odra2::event::EventError>
                where
                    T: odra2::types::FromBytes + odra2::casper_event_standard::EventInstance,
                {
                    self.env.get_event(&self.address, index)
                }

                pub fn try_total_supply(&self) -> Result<U256, OdraError> {
                    self.env.call_contract(
                        self.address,
                        odra2::CallDef::new(
                            String::from("total_supply"),
                            odra2::types::RuntimeArgs::new(),
                        ),
                    )
                }

                pub fn total_supply(&self) -> U256 {
                    self.try_total_supply().unwrap()
                }
            }
        };
        let actual = HostRefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
