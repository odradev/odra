use crate::ir::FnIR;
use proc_macro2::TokenStream;
use quote::quote;

pub fn host_try_function_item(fun: &FnIR) -> syn::ItemFn {
    let signature = try_function_signature(fun);
    let call_def_expr = call_def_with_amount(fun);

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
    let runtime_args = quote!({
        let mut named_args = odra::types::RuntimeArgs::new();
        #args
        named_args
    });
    syn::parse_quote!(odra::CallDef::new(String::from(#fun_name_str), #runtime_args))
}

fn call_def_with_amount(fun: &FnIR) -> syn::Expr {
    let fun_name_str = fun.name_str();
    let args = args_token_stream(fun);
    let runtime_args = quote!({
        let mut named_args = odra::types::RuntimeArgs::new();
        if self.attached_value > odra::types::U512::zero() {
            let _ = named_args.insert("amount", self.attached_value);
        }
        #args
        named_args
    });
    syn::parse_quote!(odra::CallDef::new(String::from(#fun_name_str), #runtime_args).with_amount(self.attached_value))
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
    fun
        .arg_names()
        .iter()
        .map(|i| quote!(let _ = named_args.insert(stringify!(#i), #i);))
        .collect::<TokenStream>()
}
