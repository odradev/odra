use crate::{ir::ModuleIR, utils};
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

impl TryFrom<&'_ ModuleIR> for HostRefStructItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let vis_pub = utils::syn::visibility_pub();

        let address = utils::ident::address();
        let env = utils::ident::env();
        let attached_value = utils::ident::attached_value();

        let ty_address = utils::ty::address();
        let ty_host_env = utils::ty::host_env();
        let ty_u512 = utils::ty::u512();

        let named_fields: syn::FieldsNamed = parse_quote!({
            #vis_pub #address: #ty_address,
            #vis_pub #env: #ty_host_env,
            #vis_pub #attached_value: #ty_u512
        });
        Ok(Self {
            vis: vis_pub,
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
    with_tokens_fn: WithTokensFnItem,
    #[syn(in = brace_token)]
    get_event_fn: GetEventFnItem,
    #[syn(in = brace_token)]
    last_call_fn: LastCallFnItem,
    #[syn(in = brace_token)]
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    functions: Vec<syn::ItemFn>
}

impl TryFrom<&'_ ModuleIR> for HostRefImplItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            ref_ident: module.host_ref_ident()?,
            brace_token: Default::default(),
            with_tokens_fn: WithTokensFnItem,
            get_event_fn: GetEventFnItem,
            last_call_fn: LastCallFnItem,
            functions: module
                .host_functions()
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

#[derive(syn_derive::ToTokens)]
pub struct HostRefItem {
    struct_item: HostRefStructItem,
    impl_item: HostRefImplItem
}

impl TryFrom<&'_ ModuleIR> for HostRefItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        Ok(Self {
            struct_item: module.try_into()?,
            impl_item: module.try_into()?
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
                pub address: odra::Address,
                pub env: odra::HostEnv,
                pub attached_value: odra::U512
            }

            impl Erc20HostRef {
                pub fn with_tokens(&self, tokens: odra::U512) -> Self {
                    Self {
                        address: self.address,
                        env: self.env.clone(),
                        attached_value: tokens
                    }
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

                pub fn try_total_supply(&self) -> Result<U256, odra::types::OdraError> {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            String::from("total_supply"),
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
            }
        };
        let actual = HostRefItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
