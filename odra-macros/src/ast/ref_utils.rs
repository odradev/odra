use crate::{
    ir::FnIR,
    utils::{self, syn::visibility_pub}
};

pub fn host_try_function_item(fun: &FnIR) -> syn::ItemFn {
    let signature = try_function_signature(fun);
    let call_def_expr = call_def_with_amount(fun);

    env_call(signature, call_def_expr)
}

pub fn host_function_item(fun: &FnIR) -> syn::ItemFn {
    let pub_vis = visibility_pub();
    let signature = function_signature(fun);
    let try_func_name = fun.try_name();
    let args = fun.arg_names();
    syn::parse_quote!(
        #pub_vis #signature {
            self.#try_func_name(#(#args),*).unwrap()
        }
    )
}

pub fn contract_function_item(fun: &FnIR) -> syn::ItemFn {
    let signature = function_signature(fun);
    let call_def_expr = call_def(fun);

    env_call(signature, call_def_expr)
}

fn env_call(sig: syn::Signature, call_def_expr: syn::Expr) -> syn::ItemFn {
    let pub_vis = visibility_pub();
    let m_env = utils::member::env();
    let m_address = utils::member::address();

    syn::parse_quote!(
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
    let args_block = runtime_args_block(fun);
    syn::parse_quote!(#ty_call_def::new(String::from(#fun_name_str), #args_block))
}

fn call_def_with_amount(fun: &FnIR) -> syn::Expr {
    let ty_call_def = utils::ty::call_def();
    let fun_name_str = fun.name_str();
    let args_block = runtime_args_with_amount_block(fun);
    let attached_value = utils::member::attached_value();

    syn::parse_quote!(#ty_call_def::new(String::from(#fun_name_str), #args_block).with_amount(#attached_value))
}

fn function_signature(fun: &FnIR) -> syn::Signature {
    let fun_name = fun.name();
    let args = fun.typed_args();
    let return_type = fun.return_type();
    let mutability = fun.is_mut().then(|| quote::quote!(mut));

    syn::parse_quote!(fn #fun_name(& #mutability self #(, #args)*) #return_type)
}

fn try_function_signature(fun: &FnIR) -> syn::Signature {
    let fun_name = fun.try_name();
    let args = fun.typed_args();
    let return_type = fun.try_return_type();
    let mutability = fun.is_mut().then(|| quote::quote!(mut));

    syn::parse_quote!(fn #fun_name(& #mutability self #(, #args)*) #return_type)
}

pub fn runtime_args_block(fun: &FnIR) -> syn::Block {
    let runtime_args = utils::expr::new_runtime_args();
    let args = utils::ident::named_args();
    let insert_args = insert_args_stmts(fun);

    syn::parse_quote!({
        let mut #args = #runtime_args;
        #(#insert_args)*
        #args
    })
}

pub fn runtime_args_with_amount_block(fun: &FnIR) -> syn::Block {
    let runtime_args = utils::expr::new_runtime_args();
    let args = utils::ident::named_args();
    let insert_amount = insert_amount_arg_stmt();
    let insert_args = insert_args_stmts(fun);

    syn::parse_quote!({
        let mut #args = #runtime_args;
        #insert_amount
        #(#insert_args)*
        #args
    })
}

fn insert_args_stmts(fun: &FnIR) -> Vec<syn::Stmt> {
    fun.arg_names()
        .iter()
        .map(insert_arg_stmt)
        .collect::<Vec<_>>()
}

fn insert_arg_stmt(ident: &syn::Ident) -> syn::Stmt {
    let name = ident.to_string();
    let args = utils::ident::named_args();

    syn::parse_quote!(let _ = #args.insert(#name, #ident);)
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
