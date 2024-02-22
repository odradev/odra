use crate::{
    ast::fn_utils,
    ir::{FnArgIR, FnIR},
    utils::{self, syn::visibility_pub}
};
use syn::{parse_quote, Attribute};

pub fn host_try_function_item(fun: &FnIR) -> syn::ItemFn {
    let signature = try_function_signature(fun);
    let call_def_expr = call_def_with_amount(fun);
    let mut docs = function_docs(fun);
    docs.push(parse_quote!(#[doc = " Does not fail in case of error, returns `odra::OdraResult` instead."]));

    env_call(signature, call_def_expr, docs)
}

pub fn host_function_item(fun: &FnIR) -> syn::ItemFn {
    let pub_vis = visibility_pub();
    let docs = function_docs(fun);
    let signature = function_signature(fun);
    let try_func_name = fun.try_name();
    let args = fun.arg_names();
    syn::parse_quote!(
        #(#docs)*
        #pub_vis #signature {
            self.#try_func_name(#(#args),*).unwrap()
        }
    )
}

pub fn contract_function_item(fun: &FnIR) -> syn::ItemFn {
    let signature = function_signature(fun);
    let call_def_expr = call_def(fun);
    let docs = function_docs(fun);

    env_call(signature, call_def_expr, docs)
}

fn env_call(sig: syn::Signature, call_def_expr: syn::Expr, docs: Vec<Attribute>) -> syn::ItemFn {
    let pub_vis = visibility_pub();
    let m_env = utils::member::env();
    let m_address = utils::member::address();

    syn::parse_quote!(
        #(#docs)*
        #pub_vis #sig {
            #m_env.call_contract(
                #m_address,
                #call_def_expr
            )
        }
    )
}

fn call_def(fun: &FnIR) -> syn::Expr {
    let ty_call_def = utils::ty::call_def();
    let fun_name_str = fun.name_str();
    let args_block = fn_utils::runtime_args_block(fun, insert_arg_stmt);
    let is_mut = fun.is_mut();
    syn::parse_quote!(#ty_call_def::new(String::from(#fun_name_str), #is_mut, #args_block))
}

fn call_def_with_amount(fun: &FnIR) -> syn::Expr {
    let ty_call_def = utils::ty::call_def();
    let fun_name_str = fun.name_str();
    let args_block = runtime_args_with_amount_block(fun, insert_arg_stmt);
    let is_mut = fun.is_mut();
    let attached_value = utils::member::attached_value();

    syn::parse_quote!(#ty_call_def::new(String::from(#fun_name_str), #is_mut, #args_block).with_amount(#attached_value))
}

fn function_signature(fun: &FnIR) -> syn::Signature {
    let fun_name = fun.name();
    let args = fun.typed_args();
    let return_type = fun.return_type();
    let mutability = fun.is_mut().then(|| quote::quote!(mut));

    syn::parse_quote!(fn #fun_name(& #mutability self #(, #args)*) #return_type)
}

fn function_docs(fun: &FnIR) -> Vec<syn::Attribute> {
    fun.attributes()
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .cloned()
        .collect()
}

fn try_function_signature(fun: &FnIR) -> syn::Signature {
    let fun_name = fun.try_name();
    let args = fun.typed_args();
    let return_type = fun.try_return_type();
    let mutability = fun.is_mut().then(|| quote::quote!(mut));

    syn::parse_quote!(
        fn #fun_name(& #mutability self #(, #args)*) #return_type)
}

fn runtime_args_with_amount_block<F: FnMut(&FnArgIR) -> syn::Stmt>(
    fun: &FnIR,
    insert_arg_fn: F
) -> syn::Block {
    let runtime_args = utils::expr::new_runtime_args();
    let args = utils::ident::named_args();
    let insert_amount = insert_amount_arg_stmt();
    let insert_args = fn_utils::insert_args_stmts(fun, insert_arg_fn);

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

    syn::parse_quote!(let _ = #args.insert(#name, #ident.clone());)
}
