use std::vec;

use quote::ToTokens;
use syn::parse_quote;
use syn::punctuated::Punctuated;

use crate::ir::FnIR;
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
        .chain(vec![ir.factory_fn()].iter())
        .map(|f| {
            let ident = f.name_str();
            let args = entrypoint_args(f)?;
            let is_mut = f.is_mut();
            let ret = match f.return_type() {
                syn::ReturnType::Default => utils::expr::unit_cl_type(),
                syn::ReturnType::Type(_, ty) => utils::expr::as_cl_type(&ty)
            };
            
            let ty = f
                .is_restricted()
                .then(utils::ty::entry_point_def_ty_constructor)
                .unwrap_or_else(utils::ty::entry_point_def_ty_public);
            let is_payable_attr = f.is_payable().then(utils::ty::entry_point_def_attr_payable);
            let is_non_reentrant = f
                .is_non_reentrant()
                .then(utils::ty::entry_point_def_attr_non_reentrant);
            let attributes = vec![is_payable_attr, is_non_reentrant]
                .into_iter()
                .flatten()
                .collect::<Punctuated<_, syn::token::Comma>>();
            let attributes = utils::expr::vec(attributes);
            let ty_entrypoint = utils::ty::entry_point_def();
            let name = utils::expr::string_from(ident);

            let expr: syn::Expr = parse_quote!(#ty_entrypoint {
                name: #name,
                args: #args,
                is_mutable: #is_mut,
                return_ty: #ret,
                ty: #ty,
                attributes: #attributes
            });
            Ok(expr)
        })
        .collect::<syn::Result<syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>>>()?;
    Ok(utils::expr::vec(struct_entrypoints))
}

fn entrypoint_args(f: &FnIR) -> syn::Result<syn::Expr> {
    let args = if f.is_constructor() {
        vec![]
    } else {
        f.named_args()
    }
    .iter()
    .map(|arg: &crate::ir::FnArgIR| {
        let ident = arg.name_str()?;
        let ty = utils::ty::unreferenced_ty(&arg.ty()?);
        Ok(utils::expr::into_arg(ty, ident))
    })
    .collect::<syn::Result<Punctuated<syn::Expr, syn::token::Comma>>>()?;
    Ok(utils::expr::vec(args))
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
                        odra::contract_def::Entrypoint {
                            name: odra::prelude::string::String::from("init"),
                            args: odra::prelude::vec![],
                            is_mutable: true,
                            return_ty: <() as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Constructor,
                            attributes: odra::prelude::vec![]
                        },
                        odra::contract_def::Entrypoint {
                            name: odra::prelude::string::String::from("factory"),
                            args: odra::prelude::vec![
                                odra::args::odra_argument::<String>("contract_name"),
                                odra::args::odra_argument::<u32>("value")
                            ],
                            is_mutable: true,
                            return_ty: <(odra::prelude::Address, odra::casper_types::URef) as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes: odra::prelude::vec![]
                        }
                    ]
                }
            }
        );
        let actual = FactoryHasEntrypointsImplItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
