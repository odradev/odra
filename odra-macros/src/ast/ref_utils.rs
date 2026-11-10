//! The functions of the contract ref and the host ref: one per entry point, each a call to
//! the contract with the arguments as runtime args. Built once here for the module and the
//! factory refs.

use crate::ir::FnType;
use crate::utils::syn::visibility_default;
use crate::{
    ir::{FnArgIR, FnIR},
    utils::{self, syn::visibility_pub}
};
use quote::format_ident;
use syn::{parse_quote, Attribute, Visibility};

pub fn host_try_function_item(fun: &FnIR) -> syn::ItemFn {
    try_function_item(fun, call_def_with_amount(fun))
}

pub fn factory_try_function_item(fun: &FnIR) -> syn::ItemFn {
    try_function_item(fun, factory_call_def_with_amount(fun))
}

/// `pub fn try_<name>(..) -> OdraResult<Ret>`, calling the contract with `call_def_expr`.
fn try_function_item(fun: &FnIR, call_def_expr: syn::Expr) -> syn::ItemFn {
    let signature = try_function_signature(fun);
    let mut attrs = function_filtered_attrs(fun);
    attrs.push(parse_quote!(#[doc = " Does not fail in case of error, returns `odra::OdraResult` instead."]));

    env_call(signature, call_def_expr, attrs, visibility_pub())
}

pub fn host_function_item(fun: &FnIR, is_trait_impl: bool) -> syn::ItemFn {
    let pub_vis = match is_trait_impl {
        true => None,
        false => Some(visibility_pub())
    };
    let attrs = function_filtered_attrs(fun);
    let signature = function_signature(fun);
    let try_func_name = fun.try_name();
    let args = fun.args().iter().map(FnArgIR::name);
    syn::parse_quote!(
        #(#attrs)*
        #pub_vis #signature {
            self.#try_func_name(#(#args),*).unwrap()
        }
    )
}

pub fn host_mut_no_ret_function_item(fun: &FnIR) -> syn::ItemFn {
    let mut attrs = function_filtered_attrs(fun);
    attrs.push(parse_quote!(#[doc = " Ignores the result of the call."]));
    let signature = function_signature(fun);
    let signature = syn::Signature {
        ident: format_ident!("{}_no_ret", fun.name()),
        output: syn::ReturnType::Default,
        ..signature
    };
    let try_func_name = fun.try_no_ret_name();
    let args = fun.args().iter().map(FnArgIR::name);
    syn::parse_quote!(
        #(#attrs)*
        pub #signature {
            self.#try_func_name(#(#args),*).unwrap()
        }
    )
}

pub fn host_mut_try_no_ret_function_item(fun: &FnIR) -> syn::ItemFn {
    let mut attrs = function_filtered_attrs(fun);
    attrs.push(parse_quote!(#[doc = " Ignores the result of the call."]));
    let signature = function_signature(fun);
    let signature = syn::Signature {
        ident: fun.try_no_ret_name(),
        output: syn::parse_quote!(-> OdraResult<()>),
        ..signature
    };
    let call_def_expr = call_def_with_amount(fun);
    env_call(signature, call_def_expr, attrs, visibility_pub())
}

pub fn contract_function_item(fun: &FnIR, is_trait_impl: bool) -> syn::ItemFn {
    contract_function(fun, is_trait_impl, call_def_with_amount(fun))
}

pub fn factory_contract_function_item(fun: &FnIR, is_trait_impl: bool) -> syn::ItemFn {
    contract_function(fun, is_trait_impl, factory_call_def_with_amount(fun))
}

/// `fn <name>(..) -> Ret`, calling the contract with `call_def_expr`; `pub` unless it
/// implements a trait.
fn contract_function(fun: &FnIR, is_trait_impl: bool, call_def_expr: syn::Expr) -> syn::ItemFn {
    let vis = match is_trait_impl {
        true => visibility_default(),
        false => visibility_pub()
    };
    let signature = function_signature(fun);
    let attrs = function_filtered_attrs(fun);

    env_call(signature, call_def_expr, attrs, vis)
}

fn env_call(
    sig: syn::Signature,
    call_def_expr: syn::Expr,
    docs: Vec<Attribute>,
    vis: Visibility
) -> syn::ItemFn {

    syn::parse_quote!(
        #(#docs)*
        #vis #sig {
            self.env.call_contract(
                self.address,
                #call_def_expr
            )
        }
    )
}

fn call_def_with_amount(fun: &FnIR) -> syn::Expr {
    let fun_name_str = fun.name_str();
    let args_block = runtime_args_with_amount_block(fun);
    let is_mut = fun.is_mut();
    let fun_name = string_from(fun_name_str);

    syn::parse_quote!(odra::CallDef::new(#fun_name, #is_mut, #args_block).with_amount(self.attached_value))
}

fn factory_call_def_with_amount(fun: &FnIR) -> syn::Expr {
    let fun_name_str = fun.name_str();
    let new_runtime_args = new_runtime_args();
    let args = utils::ident::named_args();
    let is_mut = fun.is_mut();
    let fun_name = string_from(fun_name_str);
    let fn_args = fun.args().iter().map(insert_arg_stmt).collect::<Vec<_>>();
    // The flags that tell `call` on the child it is being upgraded, or installed.
    let upgrade_cfg = quote::quote! {
        odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_is_factory_upgrade", &mut #args);
        odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut #args);
        odra::args::EntrypointArgument::insert_runtime_arg(false, "odra_cfg_create_upgrade_group", &mut #args);
    };
    let install_cfg = quote::quote! {
        odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_is_upgradable", &mut #args);
        odra::args::EntrypointArgument::insert_runtime_arg(false, "odra_cfg_is_upgrade", &mut #args);
        odra::args::EntrypointArgument::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut #args);
        odra::args::EntrypointArgument::insert_runtime_arg(contract_name.clone(), "odra_cfg_package_hash_key_name", &mut #args);
    };
    let inserts = match fun.fn_type() {
        FnType::FactoryUpgrader => quote::quote!(#(#fn_args)* #upgrade_cfg),
        FnType::FactoryBatchUpgrader => quote::quote! {
            odra::args::EntrypointArgument::insert_runtime_arg(odra::args::BatchUpgradeArgs::from(args), "args", &mut #args);
            #upgrade_cfg
        },
        _ => quote::quote!(#install_cfg #(#fn_args)*)
    };
    syn::parse_quote!(odra::CallDef::new(#fun_name, #is_mut, {
        let mut #args = #new_runtime_args;
        #inserts
        #args
    }))
}

fn function_signature(fun: &FnIR) -> syn::Signature {
    let fun_name = fun.name();
    let args = fun.args().iter().map(FnArgIR::raw);
    let return_type = fun.return_type();
    let generics = fun.generics();
    let mutability = fun.is_mut().then(|| quote::quote!(mut));

    syn::parse_quote!(fn #fun_name #generics(& #mutability self #(, #args)*) #return_type)
}

fn function_filtered_attrs(fun: &FnIR) -> Vec<syn::Attribute> {
    fun.attributes()
        .iter()
        .filter(|attr| !attr.path().is_ident("odra"))
        .cloned()
        .collect()
}

fn try_function_signature(fun: &FnIR) -> syn::Signature {
    let fun_name = fun.try_name();
    let args = fun.args().iter().map(FnArgIR::raw);
    let return_type = fun.try_return_type();
    let generics = fun.generics();
    let mutability = fun.is_mut().then(|| quote::quote!(mut));

    syn::parse_quote!(
        fn #fun_name #generics(& #mutability self #(, #args)*) #return_type)
}

/// `{ let mut args = RuntimeArgs::new(); <amount, if any>; <one insert per argument>; args }`.
fn runtime_args_with_amount_block(fun: &FnIR) -> syn::Block {
    let runtime_args = new_runtime_args();
    let args = utils::ident::named_args();
    let insert_amount = insert_amount_arg_stmt();
    let insert_args = fun.args().iter().map(insert_arg_stmt);

    syn::parse_quote!({
        let mut #args = #runtime_args;
        #insert_amount
        #(#insert_args)*
        #args
    })
}

fn insert_amount_arg_stmt() -> syn::Stmt {
    let ident = utils::ident::named_args();
    let zero = u512_zero();

    syn::parse_quote!(
        if self.attached_value > #zero {
            let _ = #ident.insert("amount", self.attached_value);
        }
    )
}

fn insert_arg_stmt(arg: &FnArgIR) -> syn::Stmt {
    let ident = arg.name_spanned_as_type();
    let name = arg.name_str();
    let args = utils::ident::named_args();
    syn::parse_quote!(odra::args::EntrypointArgument::insert_runtime_arg(#ident, #name, &mut #args);)
}

fn new_runtime_args() -> syn::Expr {
    parse_quote!(odra::casper_types::RuntimeArgs::new())
}
fn u512_zero() -> syn::Expr {
    parse_quote!(odra::casper_types::U512::zero())
}
fn string_from(string: String) -> syn::Expr {
    parse_quote!(odra::prelude::string::String::from(#string))
}
