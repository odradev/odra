use syn::parse_quote;

use crate::ir::FnIR;
use crate::{ir::ModuleImplIR, utils};

#[derive(syn_derive::ToTokens)]
pub struct HasEntrypointsImplItem {
    impl_token: syn::token::Impl,
    has_ident_ty: syn::Type,
    for_token: syn::token::For,
    module_ident: syn::Ident,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    events_fn: EntrypointsFnItem
}

impl TryFrom<&'_ ModuleImplIR> for HasEntrypointsImplItem {
    type Error = syn::Error;

    fn try_from(struct_ir: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            has_ident_ty: utils::ty::has_entrypoints(),
            for_token: Default::default(),
            module_ident: struct_ir.module_ident()?,
            brace_token: Default::default(),
            events_fn: struct_ir.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
pub struct EntrypointsFnItem {
    sig: syn::Signature,
    #[syn(braced)]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    expr: syn::Expr
}

impl TryFrom<&'_ ModuleImplIR> for EntrypointsFnItem {
    type Error = syn::Error;

    fn try_from(struct_ir: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let ident_entrypoints = utils::ident::entrypoints();
        let entrypoint_ty = utils::ty::entry_point_def();
        let expr = struct_entrypoints_expr(struct_ir)?;

        Ok(Self {
            sig: parse_quote!(fn #ident_entrypoints() -> Vec<#entrypoint_ty>),
            brace_token: Default::default(),
            expr
        })
    }
}

fn struct_entrypoints_expr(ir: &ModuleImplIR) -> Result<syn::Expr, syn::Error> {
    let struct_entrypoints = ir
        .functions()
        .iter()
        .map(|f| {
            let ident = f.name_str();
            let args = entrypoint_args(f)?;
            let is_mut = f.is_mut();
            let ret = match f.return_type() {
                syn::ReturnType::Default => utils::expr::unit_cl_type(),
                syn::ReturnType::Type(_, ty) => utils::expr::as_cl_type(&ty)
            };
            let ty = f
                .is_constructor()
                .then(utils::ty::entry_point_def_ty_constructor)
                .unwrap_or_else(utils::ty::entry_point_def_ty_public);
            let is_payable_attr = f.is_payable().then(utils::ty::entry_point_def_attr_payable);
            let is_non_reentrant = f
                .is_non_reentrant()
                .then(utils::ty::entry_point_def_attr_non_reentrant);
            let attributes = vec![is_payable_attr, is_non_reentrant]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            let ty_entrypoint = utils::ty::entry_point_def();

            let expr: syn::Expr = parse_quote!(#ty_entrypoint {
                ident: String::from(#ident),
                args: #args,
                is_mut: #is_mut,
                ret: #ret,
                ty: #ty,
                attributes: vec![#(#attributes),*]
            });
            Ok(expr)
        })
        .collect::<Result<syn::punctuated::Punctuated<syn::Expr, syn::token::Comma>, syn::Error>>(
        )?;
    Ok(parse_quote!(vec![#struct_entrypoints]))
}

fn entrypoint_args(f: &FnIR) -> Result<syn::Expr, syn::Error> {
    let args = f
        .named_args()
        .iter()
        .map(|arg| {
            let ty_arg = utils::ty::entry_point_def_arg();
            let ident = arg.name_str()?;
            let ty = utils::expr::as_cl_type(&arg.ty()?);

            let expr: syn::Expr = parse_quote!(#ty_arg {
                ident: String::from(#ident),
                ty: #ty,
                is_ref: false,
                is_slice: false
            });
            Ok(expr)
        })
        .collect::<Result<Vec<syn::Expr>, syn::Error>>()?;
    Ok(parse_quote!(vec![#(#args),*]))
}

#[cfg(test)]
mod test {
    use crate::test_utils;
    use quote::quote;

    use super::HasEntrypointsImplItem;

    #[test]
    fn test_entrypoints() {
        let module = test_utils::mock::module_impl();
        let expected = quote!(
            impl odra::contract_def::HasEntrypoints for Erc20 {
                fn entrypoints() -> Vec<odra::contract_def::Entrypoint> {
                    vec![
                        odra::contract_def::Entrypoint {
                            ident: String::from("init"),
                            args: vec![
                                odra::contract_def::Argument {
                                    ident: String::from("total_supply"),
                                    ty: <Option::<U256> as odra::casper_types::CLTyped>::cl_type(),
                                    is_ref: false,
                                    is_slice: false
                                }
                            ],
                            is_mut: true,
                            ret: <() as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Constructor,
                            attributes: vec![]
                        },
                        odra::contract_def::Entrypoint {
                            ident: String::from("total_supply"),
                            args: vec![],
                            is_mut: false,
                            ret: <U256 as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes: vec![]
                        },
                        odra::contract_def::Entrypoint {
                            ident: String::from("pay_to_mint"),
                            args: vec![],
                            is_mut: true,
                            ret: <() as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes: vec![odra::contract_def::EntrypointAttribute::Payable]
                        },
                        odra::contract_def::Entrypoint {
                            ident: String::from("approve"),
                            args: vec![
                                odra::contract_def::Argument {
                                    ident: String::from("to"),
                                    ty: <Address as odra::casper_types::CLTyped>::cl_type(),
                                    is_ref: false,
                                    is_slice: false
                                },
                                odra::contract_def::Argument {
                                    ident: String::from("amount"),
                                    ty: <U256 as odra::casper_types::CLTyped>::cl_type(),
                                    is_ref: false,
                                    is_slice: false
                                }
                            ],
                            is_mut: true,
                            ret: <() as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes: vec![odra::contract_def::EntrypointAttribute::NonReentrant]
                        }
                    ]
                }
            }
        );
        let actual = HasEntrypointsImplItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn test_trait_impl_entrypoints() {
        let module = test_utils::mock::module_trait_impl();
        let expected = quote!(
            impl odra::contract_def::HasEntrypoints for Erc20 {
                fn entrypoints() -> Vec<odra::contract_def::Entrypoint> {
                    vec![
                        odra::contract_def::Entrypoint {
                            ident: String::from("total_supply"),
                            args: vec![],
                            is_mut: false,
                            ret: <U256 as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes: vec![]
                        },
                        odra::contract_def::Entrypoint {
                            ident: String::from("pay_to_mint"),
                            args: vec![],
                            is_mut: true,
                            ret: <() as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes: vec![odra::contract_def::EntrypointAttribute::Payable]
                        }
                    ]
                }
            }
        );
        let actual = HasEntrypointsImplItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn test_delegated_entrypoints() {
        let module = test_utils::mock::module_delegation();
        let expected = quote!(
            impl odra::contract_def::HasEntrypoints for Erc20 {
                fn entrypoints() -> Vec<odra::contract_def::Entrypoint> {
                    vec![
                        odra::contract_def::Entrypoint {
                            ident: String::from("total_supply"),
                            args: vec![],
                            is_mut: false,
                            ret: <U256 as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes: vec![]
                        },
                        odra::contract_def::Entrypoint {
                            ident: String::from("get_owner"),
                            args: vec![],
                            is_mut: false,
                            ret: <Address as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes : vec![]
                        },
                        odra::contract_def::Entrypoint {
                            ident: String::from("set_owner"),
                            args: vec![
                                odra::contract_def::Argument {
                                    ident: String::from("new_owner"),
                                    ty: <Address as odra::casper_types::CLTyped>::cl_type(),
                                    is_ref: false,
                                    is_slice: false
                                }
                            ],
                            is_mut: true,
                            ret: <() as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes: vec![]
                        },
                        odra::contract_def::Entrypoint {
                            ident: String::from("name"),
                            args: vec![],
                            is_mut: false,
                            ret: <String as odra::casper_types::CLTyped >::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes : vec![]
                        },
                        odra::contract_def::Entrypoint {
                            ident: String::from("symbol"),
                            args: vec![],
                            is_mut: false,
                            ret: <String as odra::casper_types::CLTyped>::cl_type(),
                            ty: odra::contract_def::EntrypointType::Public,
                            attributes :vec![]
                        }
                    ]
                }
            }
        );
        let actual = HasEntrypointsImplItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
