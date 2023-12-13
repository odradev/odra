use crate::ast::host_ref_item::HostRefItem;
use crate::ast::parts_utils::{UsePreludeItem, UseSuperItem};
use crate::ast::ref_item::RefItem;
use crate::ast::test_parts::{PartsModuleItem, TestPartsReexportItem};
use crate::ir::ModuleImplIR;
use derive_try_from::TryFromRef;

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
pub struct ExternalContractImpl {
    #[expr(item.self_code())]
    self_code: proc_macro2::TokenStream,
    ref_item: RefItem,
    test_parts: TestPartsItem,
    test_parts_reexport: TestPartsReexportItem
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
struct TestPartsItem {
    parts_module: PartsModuleItem,
    #[syn(braced)]
    #[default]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    #[default]
    use_super: UseSuperItem,
    #[syn(in = brace_token)]
    #[default]
    use_prelude: UsePreludeItem,
    #[syn(in = brace_token)]
    host_ref: HostRefItem
}

#[cfg(test)]
mod test {
    use crate::test_utils;

    use super::ExternalContractImpl;

    #[test]
    fn external_contract_impl() {
        let ir = test_utils::mock_ext_contract();
        let item = ExternalContractImpl::try_from(&ir).unwrap();
        let expected = quote::quote! {
            pub trait Token {
                fn balance_of(&self, owner: Address) -> U256;
            }

            pub struct TokenContractRef {
                env: Rc<odra::ContractEnv>,
                address: odra::Address,
            }

            impl TokenContractRef {
                pub fn address(&self) -> &odra::Address {
                    &self.address
                }

                pub fn balance_of(&self, owner: Address) -> U256 {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            String::from("balance_of"),
                            {
                                let mut named_args = odra::RuntimeArgs::new();
                                let _ = named_args.insert("owner", owner);
                                named_args
                            }
                        ),
                    )
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            mod __token_test_parts {
                use super::*;
                use odra::prelude::*;


                pub struct TokenHostRef {
                    pub address: odra::Address,
                    pub env: odra::HostEnv,
                    pub attached_value: odra::U512
                }

                impl TokenHostRef {
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

                    pub fn try_balance_of(&self, owner: Address) -> Result<U256, odra::OdraError> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                String::from("balance_of"),
                                {
                                    let mut named_args = odra::RuntimeArgs::new();
                                    if self.attached_value > odra::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    let _ = named_args.insert("owner", owner);
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }

                    pub fn balance_of(&self, owner: Address) -> U256 {
                        self.try_balance_of(owner).unwrap()
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub use __token_test_parts::*;
        };

        test_utils::assert_eq(item, expected);
    }
}
