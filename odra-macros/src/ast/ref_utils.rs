use crate::ir::FnIR;
use proc_macro2::TokenStream;
use quote::quote;

pub fn host_try_function_item(fun: &FnIR) -> syn::ItemFn {
    let signature = try_function_signature(fun);
    let call_def_expr = call_def(fun);

    env_call(signature, call_def_expr)
}

pub fn host_function_item(fun: &FnIR) -> syn::ItemFn {
    let signature = function_signature(fun);
    let try_func_name = fun.try_name();
    let args = fun.arg_names();
    syn::parse_quote!(
        pub #signature {
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
    syn::parse_quote!(
        pub #sig {
            self.env.call_contract(
                self.address,
                #call_def_expr
            )
        }
    )
}

fn call_def(fun: &FnIR) -> syn::Expr {
    let fun_name_str = fun.name_str();
    let args = args_token_stream(fun);
    syn::parse_quote!(odra2::CallDef::new(String::from(#fun_name_str), #args))
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

fn args_token_stream(fun: &FnIR) -> TokenStream {
    let args = fun.arg_names();

    match fun.args_len() {
        0 => quote!(odra2::types::RuntimeArgs::new()),
        _ => {
            let args = args
                .iter()
                .map(|i| quote!(let _ = named_args.insert(stringify!(#i), #i);))
                .collect::<TokenStream>();
            quote!({
                let mut named_args = odra2::types::RuntimeArgs::new();
                #args
                named_args
            })
        }
    }
}
