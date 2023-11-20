use syn::parse_quote;
use crate::ir::ModuleIR;

use super::{
    host_ref_item::HostRefItem,
    parts_utils::{UsePreludeItem, UseSuperItem, self}
};

#[derive(syn_derive::ToTokens)]
pub struct PartsModuleItem {
    attr: syn::Attribute,
    mod_token: syn::token::Mod,
    ident: syn::Ident
}

impl TryFrom<&'_ ModuleIR> for PartsModuleItem {
    type Error = syn::Error;

    fn try_from(value: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let ident = parts_utils::test_parts_mod_ident(value)?;
        let attr = parse_quote!(#[cfg(not(target_arch = "wasm32"))]);
        Ok(Self {
            attr,
            mod_token: Default::default(),
            ident
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct TestParts {
    parts_module: PartsModuleItem,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    use_super: UseSuperItem,
    #[syn(in = brace_token)]
    use_prelude: UsePreludeItem,
    #[syn(in = brace_token)]
    host_ref: HostRefItem
}

impl TryFrom<&'_ ModuleIR> for TestParts {
    type Error = syn::Error;

    fn try_from(value: &'_ ModuleIR) -> Result<Self, Self::Error> {
        Ok(TestParts {
            parts_module: value.try_into()?,
            brace_token: Default::default(),
            use_prelude: UsePreludeItem,
            use_super: UseSuperItem,
            host_ref: value.try_into()?
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{self, mock_module};

    #[test]
    fn test_parts() {
        let module = mock_module();
        let actual = TestParts::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[cfg(not(target_arch = "wasm32"))]
            mod __erc20_test_parts {
                use super::*;
                use odra2::prelude::*;

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
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}
