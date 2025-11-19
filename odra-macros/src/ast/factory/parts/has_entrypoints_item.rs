use std::vec;

use quote::ToTokens;
use syn::parse_quote;

use crate::ast::wasm_parts_utils;
use crate::{ir::ModuleImplIR, utils};

pub struct FactoryHasEntrypointsImplItem {
    has_ident_ty: syn::Type,
    module_ident: syn::Ident,
    events_fn: EntrypointsFnItem
}

impl TryFrom<&'_ ModuleImplIR> for FactoryHasEntrypointsImplItem {
    type Error = syn::Error;

    fn try_from(struct_ir: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            has_ident_ty: utils::ty::has_entrypoints(),
            module_ident: struct_ir.module_ident()?,
            events_fn: struct_ir.try_into()?
        })
    }
}

impl ToTokens for FactoryHasEntrypointsImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let has_ident_ty = &self.has_ident_ty;
        let module_ident = &self.module_ident;
        let events_fn = &self.events_fn;

        tokens.extend(quote::quote! {
            impl #has_ident_ty for #module_ident {
                #events_fn
            }
        });
    }
}

pub struct EntrypointsFnItem {
    fn_ident: syn::Ident,
    ret_ty: syn::Type,
    expr: syn::Expr
}

impl TryFrom<&'_ ModuleImplIR> for EntrypointsFnItem {
    type Error = syn::Error;

    fn try_from(struct_ir: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let entrypoint_ty = utils::ty::entry_point_def();
        let expr = struct_entrypoints_expr(struct_ir)?;

        Ok(Self {
            fn_ident: utils::ident::entrypoints(),
            ret_ty: utils::ty::vec_of(&entrypoint_ty),
            expr
        })
    }
}

impl ToTokens for EntrypointsFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fn_ident = &self.fn_ident;
        let ret_ty = &self.ret_ty;
        let expr = &self.expr;

        tokens.extend(quote::quote! {
            fn #fn_ident() -> #ret_ty {
                #expr
            }
        });
    }
}

fn struct_entrypoints_expr(ir: &ModuleImplIR) -> syn::Result<syn::Expr> {
    let struct_entrypoints = vec![ir.constructor()]
        .iter()
        .filter_map(|f| f.as_ref())
        .chain(vec![ir.factory_fn(), ir.factory_upgrade_fn(), ir.factory_batch_upgrade_fn()].iter())
        .map(|f| {
            let args = wasm_parts_utils::param_parameters(f);
            let expr = match f.fn_type() {
                crate::ir::FnType::Constructor => utils::expr::constructor_ep(parse_quote!(odra::prelude::vec![])),
                crate::ir::FnType::Upgrader => utils::expr::upgrader_ep(args),
                crate::ir::FnType::Factory => utils::expr::factory_ep(args),
                crate::ir::FnType::FactoryUpgrader => utils::expr::factory_upgrade_ep(args),
                crate::ir::FnType::FactoryBatchUpgrader => utils::expr::factory_batch_upgrade_ep(),
                crate::ir::FnType::Regular => utils::expr::regular_ep(
                    f.name_str(),
                    args,
                    wasm_parts_utils::param_ret_ty(f),
                    f.is_payable(),
                    f.is_non_reentrant()
                ),
            };
            Ok(expr)
        })
        .collect::<syn::Result<syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>>>()?;
    Ok(utils::expr::vec_try_into(struct_entrypoints))
}

#[cfg(test)]
mod test {
    use crate::test_utils;
    use quote::quote;

    use super::FactoryHasEntrypointsImplItem;

    #[test]
    fn test_entrypoints() {
        let module = test_utils::mock::module_factory_impl();
        let expected = quote!(
            impl odra::contract_def::HasEntrypoints for Erc20Factory {
                fn entrypoints() -> odra::prelude::vec::Vec<odra::contract_def::Entrypoint> {
                    odra::prelude::vec![
                        odra::entry_point::EntryPoint::Constructor {
                            args: odra::prelude::vec![]
                        },
                        odra::entry_point::EntryPoint::Factory {
                           args: vec![
                                odra::args::parameter::<odra::prelude::string::String>("contract_name"),
                                odra::args::parameter::<u32>("value")
                            ],
                        },
                        odra::entry_point::EntryPoint::FactoryUpgrade {
                           args: vec![
                                odra::args::parameter::<odra::prelude::string::String>("contract_name")
                            ],
                        },
                        odra::entry_point::EntryPoint::FactoryBatchUpgrade
                    ].into_iter().map(TryInto::try_into).collect::<Result<_, _>>().unwrap_or_default()
                }
            }
        );
        let actual = FactoryHasEntrypointsImplItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
