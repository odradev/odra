use derive_try_from::TryFromRef;
use syn::parse_quote;

use crate::{ir::ModuleIR, utils};

use super::parts_utils::{UsePreludeItem, UseSuperItem};

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleIR)]
pub struct BlueprintItem {
    #[expr(utils::attr::odra_module(&item.module_str()?))]
    attr: syn::Attribute,
    mod_item: BlueprintModItem
}

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleIR)]
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
    #[default]
    #[syn(in = brace_token)]
    use_prelude: UsePreludeItem,
    #[syn(in = brace_token)]
    schema_fn: SchemaFnItem
}

#[derive(syn_derive::ToTokens)]
struct SchemaFnItem {
    no_mangle_attr: syn::Attribute,
    sig: syn::Signature,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    expr: syn::Expr
}

impl TryFrom<&'_ ModuleIR> for SchemaFnItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleIR) -> Result<Self, Self::Error> {
        let ty_module = utils::ty::from_ident(&module.module_ident()?);
        let ty_blueprint = utils::ty::contract_blueprint();

        let ident_name = utils::ident::name();
        let ident_events = utils::ident::events();
        let ident_entrypoints = utils::ident::entrypoints();
        let ident_module_schema = utils::ident::module_schema();

        let module_name = module.module_str()?;
        let expr_events = utils::expr::events(&ty_module);
        let expr_entrypoints = utils::expr::entrypoints(&ty_module);

        let expr = parse_quote!(#ty_blueprint {
            #ident_name: #module_name,
            #ident_events: #expr_events,
            #ident_entrypoints: #expr_entrypoints
        });

        Ok(Self {
            no_mangle_attr: utils::attr::no_mangle(),
            sig: parse_quote!(fn #ident_module_schema() -> #ty_blueprint),
            brace_token: Default::default(),
            expr
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
        let module = test_utils::mock_module();
        let item = BlueprintItem::try_from(&module).unwrap();
        let expected = quote!(
            #[cfg(odra_module = "Erc20")]
            mod __erc20_schema {
                use super::*;
                use odra::prelude::*;

                #[no_mangle]
                fn module_schema() -> odra::contract_def::ContractBlueprint {
                    odra::contract_def::ContractBlueprint {
                        name: "Erc20",
                        events: <Erc20 as odra::contract_def::HasEvents>::events(),
                        entrypoints: <Erc20 as odra::contract_def::HasEntrypoints>::entrypoints()
                    }
                }
            }
        );
        test_utils::assert_eq(item, &expected);
    }
}
