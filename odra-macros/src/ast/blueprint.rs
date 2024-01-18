use derive_try_from::TryFromRef;
use syn::parse_quote;

use crate::{ir::ModuleImplIR, utils};

use super::parts_utils::UseSuperItem;

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
pub struct BlueprintItem {
    #[expr(utils::attr::odra_module(&item.module_str()?))]
    attr: syn::Attribute,
    mod_item: BlueprintModItem
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
struct BlueprintModItem {
    #[default]
    mod_token: syn::token::Mod,
    #[expr(item.schema_mod_ident()?)]
    ident: syn::Ident,
    #[syn(braced)]
    #[default]
    brace_token: syn::token::Brace,
    #[default]
    #[syn(in = brace_token)]
    use_super: UseSuperItem,
    #[syn(in = brace_token)]
    schema_fn: SchemaFnItem
}

#[derive(syn_derive::ToTokens)]
struct SchemaFnItem {
    no_mangle_attr: syn::Attribute,
    // not_wasm32_attr: syn::Attribute,
    sig: syn::Signature,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    expr: syn::Expr
}

impl TryFrom<&'_ ModuleImplIR> for SchemaFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let ty_blueprint = utils::ty::contract_blueprint();
        let ident_module_schema = utils::ident::module_schema();

        Ok(Self {
            no_mangle_attr: utils::attr::no_mangle(),
            // not_wasm32_attr: utils::attr::not_wasm32(),
            sig: parse_quote!(fn #ident_module_schema() -> #ty_blueprint),
            brace_token: Default::default(),
            expr: utils::expr::new_blueprint(&module.module_ident()?)
        })
    }
}

#[cfg(test)]
mod test {
    use super::BlueprintItem;
    use crate::test_utils;
    use quote::quote;

    #[test]
    fn blueprint_item() {
        let module = test_utils::mock::module_impl();
        let item = BlueprintItem::try_from(&module).unwrap();
        let expected = quote!(
            #[cfg(odra_module = "Erc20")]
            mod __erc20_schema {
                use super::*;

                #[no_mangle]
                #[cfg(not(target_arch = "wasm32"))]
                fn module_schema() -> odra::contract_def::ContractBlueprint {
                    odra::contract_def::ContractBlueprint::new::<Erc20>()
                }
            }
        );
        test_utils::assert_eq(item, &expected);
    }
}
