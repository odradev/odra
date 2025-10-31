use quote::{ToTokens, TokenStreamExt};

use crate::ModuleImplIR;

pub struct FactoryContractItem {
    module_ident: syn::Ident,
    host_ref: syn::Ident,
    contract_ref: syn::Ident
}

impl TryFrom<&'_ ModuleImplIR> for FactoryContractItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: module.module_ident()?,
            host_ref: module.host_ref_ident()?,
            contract_ref: module.contract_ref_ident()?
        })
    }
}

impl ToTokens for FactoryContractItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;
        let host_ref = &self.host_ref;
        let contract_ref = &self.contract_ref;
        tokens.append_all(quote::quote! {
            impl odra::OdraContract for #module_ident {
                #[cfg(not(target_arch = "wasm32"))]
                type HostRef = #host_ref;

                type ContractRef = #contract_ref;

                #[cfg(not(target_arch = "wasm32"))]
                type InitArgs = odra::host::NoArgs;

                #[cfg(not(target_arch = "wasm32"))]
                type UpgradeArgs = odra::host::FactoryUpgradeArgs;
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[test]
    fn test_contract_item() {
        let module = test_utils::mock::module_factory_impl();

        let item = FactoryContractItem::try_from(&module).expect("Failed to create ContractItem");
        let expected = quote::quote! {
            impl odra::OdraContract for Erc20Factory {
                #[cfg(not(target_arch = "wasm32"))]
                type HostRef = Erc20FactoryHostRef;

                type ContractRef = Erc20FactoryContractRef;

                #[cfg(not(target_arch = "wasm32"))]
                type InitArgs = odra::host::NoArgs;

                #[cfg(not(target_arch = "wasm32"))]
                type UpgradeArgs = odra::host::FactoryUpgradeArgs;
            }
        };
        test_utils::assert_eq(item, expected);
    }
}
