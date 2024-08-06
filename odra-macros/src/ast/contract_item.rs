use proc_macro2::TokenStream;
use syn::parse_quote;
use crate::ModuleImplIR;

#[derive(syn_derive::ToTokens)]
pub struct ContractItem {
    code: TokenStream
}

impl TryFrom<&'_ ModuleImplIR> for ContractItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let module_ident = module.module_ident()?;
        let host_ref = module.host_ref_ident()?;
        
        let contract_ref = module.contract_ref_ident()?;
        let has_constructor_args = module.constructor().map(|c| c.has_args()).unwrap_or_default();
        let init_args: syn::Path = match has_constructor_args {
            true => module.init_args_ident()?.into(),
            false => parse_quote!(odra::host::NoArgs)
        };
        
        Ok(Self {
            code: quote::quote! {
                impl odra::OdraContract for #module_ident {
                    #[cfg(not(target_arch = "wasm32"))]
                    type HostRef = #host_ref;
                
                    type ContractRef = #contract_ref;
                    
                    #[cfg(not(target_arch = "wasm32"))]
                    type InitArgs = #init_args;
                }
            }
        })
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[test]
    fn test_contract_item() {
        let module = test_utils::mock::module_impl();

        let item = ContractItem::try_from(&module).unwrap();

        let expected = quote::quote! { 
            impl odra::OdraContract for Erc20 {
                #[cfg(not(target_arch = "wasm32"))]
                type HostRef = Erc20HostRef;

                type ContractRef = Erc20ContractRef;

                #[cfg(not(target_arch = "wasm32"))]
                type InitArgs = Erc20InitArgs;
            }
        };
        test_utils::assert_eq(item, expected);
    }
}