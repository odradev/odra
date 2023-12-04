use syn::parse_quote;

pub fn read_runtime_arg(ident: &syn::Ident) -> syn::Stmt {
    let name_str = ident.to_string();
    parse_quote!(
        let #ident = odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::get_named_arg(#name_str);
    )
}

pub fn runtime_return(result_ident: &syn::Ident) -> syn::Stmt {
    parse_quote!(
        odra::odra_casper_wasm_env::casper_contract::contract_api::runtime::ret(
            odra::odra_casper_wasm_env::casper_contract::unwrap_or_revert::UnwrapOrRevert::unwrap_or_revert(
                odra::casper_types::CLValue::from_t(#result_ident)
            )
        );
    )
}

pub fn new_wasm_contract_env(ident: &syn::Ident) -> syn::Stmt {
    parse_quote!(
        let #ident = odra::odra_casper_wasm_env::WasmContractEnv::new_env();
    )
}

pub fn new_module(
    contract_ident: &syn::Ident,
    module_ident: &syn::Ident,
    env_ident: &syn::Ident
) -> syn::Stmt {
    parse_quote!(
        let #contract_ident = #module_ident::new(Rc::new(#env_ident));
    )
}

pub fn new_mut_module(
    contract_ident: &syn::Ident,
    module_ident: &syn::Ident,
    env_ident: &syn::Ident
) -> syn::Stmt {
    parse_quote!(
        let mut #contract_ident = #module_ident::new(Rc::new(#env_ident));
    )
}

pub fn install_contract(entry_points: syn::Expr, schemas: syn::Expr, args: syn::Expr) -> syn::Stmt {
    parse_quote!(odra::odra_casper_wasm_env::host_functions::install_contract(
        #entry_points,
        #schemas,
        #args
    );)
}
