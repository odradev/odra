use quote::{quote, ToTokens, TokenStreamExt};

use crate::{ir::ModuleIR, utils};

use super::parts_utils::{UseSuperItem, UsePreludeItem};

#[derive(syn_derive::ToTokens)]
pub struct WasmPartsModuleItem {
    #[to_tokens(|tokens, f| tokens.append_all(f))]
    attrs: Vec<syn::Attribute>,
    mod_token: syn::token::Mod,
    ident: syn::Ident,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    use_super: UseSuperItem,
    #[syn(in = braces)]
    use_prelude: UsePreludeItem,
}

impl TryFrom<&'_ ModuleIR> for WasmPartsModuleItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let module_str = module.module_str()?;
        let ident = module.wasm_parts_mod_ident()?; 
        Ok(Self { 
            attrs: vec![utils::attr::wasm32(), utils::attr::odra_module(&module_str)], 
            mod_token: Default::default(), 
            ident, 
            braces: Default::default(),
            use_super: UseSuperItem,
            use_prelude: UsePreludeItem,

        })
    }
}



#[cfg(test)]
mod test {
    use crate::test_utils;
    use super::WasmPartsModuleItem;

    #[test]
    fn test() {
        let module = test_utils::mock_module();
        let actual = WasmPartsModuleItem::try_from(&module).unwrap();

        let expected = quote::quote! {
            #[cfg(target_arch = "wasm32")]
            #[cfg(odra_module = "Erc20")]
            mod __erc20_wasm_parts {
                use super::*;
                use odra::prelude::*;
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}