use quote::ToTokens;
use syn::{parse_quote, punctuated::Punctuated};

pub fn new_runtime_args() -> syn::Expr {
    let ty = super::ty::runtime_args();
    parse_quote!(#ty::new())
}

pub fn u512_zero() -> syn::Expr {
    let ty = super::ty::u512();
    parse_quote!(#ty::zero())
}

pub fn new_parameter(name: String, ty: syn::Type) -> syn::Expr {
    let ty = super::ty::unreferenced_ty(&ty);

    parse_quote!(odra::args::parameter::<#ty>(#name))
}

pub fn as_cl_type(ty: &syn::Type) -> syn::Expr {
    let ty = super::ty::unreferenced_ty(ty);
    let ty_cl_typed = super::ty::cl_typed();
    let ty = super::syn::as_casted_ty_stream(&ty, ty_cl_typed);
    parse_quote!(#ty::cl_type())
}

pub fn unit_cl_type() -> syn::Expr {
    let ty_cl_typed = super::ty::cl_typed();
    parse_quote!(<() as #ty_cl_typed>::cl_type())
}

pub fn string_from(string: String) -> syn::Expr {
    let ty = super::ty::string();
    parse_quote!(#ty::from(#string))
}

pub fn vec<T: ToTokens>(content: T) -> syn::Expr {
    parse_quote!(odra::prelude::vec![#content])
}

pub fn new_entry_point(
    name: String,
    args: Vec<syn::PatType>,
    is_payable: bool,
    is_offchain: bool
) -> syn::Expr {
    let ty = super::ty::odra_entry_point();
    let name = string_from(name);
    let args_stream = args
        .iter()
        .map(new_entry_point_arg)
        .collect::<Punctuated<_, syn::Token![,]>>();
    let args_vec = vec(args_stream);
    if is_offchain {
        parse_quote!(#ty::new_offchain(#name, #args_vec))
    } else if is_payable {
        parse_quote!(#ty::new_payable(#name, #args_vec))
    } else {
        parse_quote!(#ty::new(#name, #args_vec))
    }
}

fn new_entry_point_arg(arg: &syn::PatType) -> syn::Expr {
    let ty = super::ty::odra_entry_point_arg();
    let arg_ty = super::ty::unreferenced_ty(&arg.ty);
    let name = string_from(arg.pat.to_token_stream().to_string());
    parse_quote!(#ty::new::<#arg_ty>(#name))
}

pub fn constructor_ep(args_expr: syn::Expr) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::Constructor {
        args: #args_expr
    })
}

pub fn upgrader_ep(args_expr: syn::Expr) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::Upgrader {
        args: #args_expr
    })
}

pub fn regular_ep(
    name: String,
    args_expr: syn::Expr,
    ret_ty_expr: syn::Expr,
    is_payable: bool,
    is_non_reentrant: bool
) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::Regular {
        name: #name,
        args: #args_expr,
        ret_ty: #ret_ty_expr,
        is_non_reentrant: #is_non_reentrant,
        is_payable: #is_payable
    })
}

pub fn factory_ep(args_expr: syn::Expr) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::Factory {
        args: #args_expr,
    })
}

pub fn factory_upgrade_ep(args_expr: syn::Expr) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::FactoryUpgrade {
        args: #args_expr,
    })
}

pub fn factory_batch_upgrade_ep() -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::FactoryBatchUpgrade)
}

pub fn template_ep(name: String, args_expr: syn::Expr, ret_ty_expr: syn::Expr) -> syn::Expr {
    parse_quote!(odra::entry_point::EntryPoint::Template {
        name: #name,
        args: #args_expr,
        ret_ty: #ret_ty_expr
    })
}
