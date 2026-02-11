use crate::ir::{FnType, ModuleImplIR};
use crate::utils::syn::visibility_default;
use crate::utils::ty;
use crate::{
    ast::fn_utils,
    ir::{FnArgIR, FnIR},
    utils::{self, syn::visibility_pub}
};
use quote::{ToTokens, format_ident};
use syn::{parse_quote, Attribute, Visibility};

pub fn host_try_function_item(fun: &FnIR) -> syn::ItemFn {
    let signature = try_function_signature(fun);
    let call_def_expr = call_def_with_amount(fun);
    let mut attrs = function_filtered_attrs(fun);
    attrs.push(parse_quote!(#[doc = " Does not fail in case of error, returns `odra::OdraResult` instead."]));

    env_call(signature, call_def_expr, attrs, visibility_pub())
}

pub fn factory_try_function_item(fun: &FnIR) -> syn::ItemFn {
    let signature = try_function_signature(fun);

    let call_def_expr = factory_call_def_with_amount(fun);
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
    let args = fun.arg_names();
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
    let args = fun.arg_names();
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
    let ret_ty = utils::ty::odra_result_unit();
    let signature = function_signature(fun);
    let signature = syn::Signature {
        ident: fun.try_no_ret_name(),
        output: syn::parse_quote!(-> #ret_ty),
        ..signature
    };
    let try_func_name = fun.try_name();
    let args = fun.arg_names();
    syn::parse_quote!(
        #(#attrs)*
        pub #signature {
            let _ = self.#try_func_name(#(#args),*)?;
            Ok(())
        }
    )
}

pub fn contract_function_item(fun: &FnIR, is_trait_impl: bool) -> syn::ItemFn {
    let vis = match is_trait_impl {
        true => visibility_default(),
        false => visibility_pub()
    };
    let signature = function_signature(fun);
    let call_def_expr = call_def_with_amount(fun);
    let attrs = function_filtered_attrs(fun);

    env_call(signature, call_def_expr, attrs, vis)
}

pub fn factory_contract_function_item(fun: &FnIR, is_trait_impl: bool) -> syn::ItemFn {
    let vis = match is_trait_impl {
        true => visibility_default(),
        false => visibility_pub()
    };
    let signature = function_signature(fun);
    let call_def_expr = factory_call_def_with_amount(fun);
    let attrs = function_filtered_attrs(fun);

    env_call(signature, call_def_expr, attrs, vis)
}

fn env_call(
    sig: syn::Signature,
    call_def_expr: syn::Expr,
    docs: Vec<Attribute>,
    vis: Visibility
) -> syn::ItemFn {
    let m_env = utils::member::env();
    let m_address = utils::member::address();

    syn::parse_quote!(
        #(#docs)*
        #vis #sig {
            #m_env.call_contract(
                #m_address,
                #call_def_expr
            )
        }
    )
}

fn call_def_with_amount(fun: &FnIR) -> syn::Expr {
    let ty_call_def = utils::ty::call_def();
    let fun_name_str = fun.name_str();
    let args_block = runtime_args_with_amount_block(fun, insert_arg_stmt);
    let is_mut = fun.is_mut();
    let attached_value = utils::member::attached_value();
    let fun_name = utils::expr::string_from(fun_name_str);

    syn::parse_quote!(#ty_call_def::new(#fun_name, #is_mut, #args_block).with_amount(#attached_value))
}

fn factory_call_def_with_amount(fun: &FnIR) -> syn::Expr {
    let ty_call_def = utils::ty::call_def();
    let fun_name_str = fun.name_str();
    let new_runtime_args = utils::expr::new_runtime_args();
    let args = utils::ident::named_args();
    let is_mut = fun.is_mut();
    let fun_name = utils::expr::string_from(fun_name_str);
    let ty_ep_arg = utils::ty::entry_point_arg();
    let fn_args = fun
        .named_args()
        .iter()
        .map(|arg| {
            let ident = arg.name().unwrap();
            let name = ident.to_string();
            quote::quote!(#ty_ep_arg::insert_runtime_arg(#ident, #name, &mut #args);)
        })
        .collect::<Vec<_>>();
    let ty_args = utils::ty::batch_upgrade_args();
    match fun.fn_type() {
        FnType::FactoryUpgrader => syn::parse_quote!(#ty_call_def::new(#fun_name, #is_mut, {
            let mut #args = #new_runtime_args;
            #(#fn_args)*
            #ty_ep_arg::insert_runtime_arg(true, "odra_cfg_is_factory_upgrade", &mut #args);
            #ty_ep_arg::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut #args);
            #ty_ep_arg::insert_runtime_arg(false, "odra_cfg_create_upgrade_group", &mut #args);
            #args
        })),
        FnType::FactoryBatchUpgrader => syn::parse_quote!(#ty_call_def::new(#fun_name, #is_mut, {
            let mut #args = #new_runtime_args;
            #ty_ep_arg::insert_runtime_arg(#ty_args::from(args), "args", &mut #args);
            #ty_ep_arg::insert_runtime_arg(true, "odra_cfg_is_factory_upgrade", &mut #args);
            #ty_ep_arg::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut #args);
            #ty_ep_arg::insert_runtime_arg(false, "odra_cfg_create_upgrade_group", &mut #args);
            #args
        })),
        _ => syn::parse_quote!(#ty_call_def::new(#fun_name, #is_mut, {
            let mut #args = #new_runtime_args;
            #ty_ep_arg::insert_runtime_arg(true, "odra_cfg_is_upgradable", &mut #args);
            #ty_ep_arg::insert_runtime_arg(false, "odra_cfg_is_upgrade", &mut #args);
            #ty_ep_arg::insert_runtime_arg(true, "odra_cfg_allow_key_override", &mut #args);
            #ty_ep_arg::insert_runtime_arg(contract_name.clone(), "odra_cfg_package_hash_key_name", &mut #args);
            #(#fn_args)*
            #args
        }))
    }
}

fn function_signature(fun: &FnIR) -> syn::Signature {
    let fun_name = fun.name();
    let args = fun.typed_args();
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
    let args = fun.typed_args();
    let return_type = fun.try_return_type();
    let generics = fun.generics();
    let mutability = fun.is_mut().then(|| quote::quote!(mut));

    syn::parse_quote!(
        fn #fun_name #generics(& #mutability self #(, #args)*) #return_type)
}

fn runtime_args_with_amount_block<F: FnMut(&FnArgIR) -> syn::Stmt>(
    fun: &FnIR,
    insert_arg_fn: F
) -> syn::Block {
    let runtime_args = utils::expr::new_runtime_args();
    let args = utils::ident::named_args();
    let insert_amount = insert_amount_arg_stmt();
    let ty = utils::ty::entry_point_arg();

    let insert_args = match fun.fn_type() {
        FnType::FactoryBatchUpgrader => {
            let args_ty = utils::ty::batch_upgrade_args();
            vec![parse_quote!(#ty::insert_runtime_arg(#args_ty::from(args), "args", &mut #args);)]
        }
        FnType::Factory => fn_utils::insert_args_stmts(fun, insert_arg_fn),
        _ => fn_utils::insert_args_stmts(fun, insert_arg_fn)
    };

    syn::parse_quote!({
        let mut #args = #runtime_args;
        #insert_amount
        #(#insert_args)*
        #args
    })
}

fn insert_amount_arg_stmt() -> syn::Stmt {
    let ident = utils::ident::named_args();
    let zero = utils::expr::u512_zero();
    let attached_value = utils::member::attached_value();

    syn::parse_quote!(
        if #attached_value > #zero {
            let _ = #ident.insert("amount", #attached_value);
        }
    )
}

pub fn insert_arg_stmt(arg: &FnArgIR) -> syn::Stmt {
    let ident = arg.name().unwrap();
    let name = ident.to_string();
    let args = utils::ident::named_args();
    let ty = ty::entry_point_arg();
    syn::parse_quote!(#ty::insert_runtime_arg(#ident, #name, &mut #args);)
}

pub struct SchemaErrorsItem {
    ident: syn::Ident
}

impl TryFrom<&ModuleImplIR> for SchemaErrorsItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            ident: ir.contract_ref_ident()?
        })
    }
}

impl ToTokens for SchemaErrorsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let item = quote::quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for #ident {}
        );
        item.to_tokens(tokens);
    }
}

pub struct SchemaEventsItem {
    ident: syn::Ident
}

impl TryFrom<&ModuleImplIR> for SchemaEventsItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            ident: ir.contract_ref_ident()?
        })
    }
}

impl ToTokens for SchemaEventsItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let item = quote::quote!(
            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for #ident {}
        );
        item.to_tokens(tokens);
    }
}
