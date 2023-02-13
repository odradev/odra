use odra_types::contract_def::{ContractDef, Entrypoint};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Path;

pub fn to_variant_branch(
    ep: &Entrypoint,
    contract_path: &Path,
    add_action_attr: bool
) -> TokenStream {
    let fn_ident = format_ident!("{}", ep.ident);
    let get_args = get_args(ep);
    let add_attrs = add_attributes(ep);
    let action_attr = if add_action_attr {
        add_action_attribute(ep)
    } else {
        quote!()
    };
    let args = ep
        .args
        .iter()
        .map(|arg| {
            let ident = format_ident!("{}", arg.ident);
            quote!(#ident,)
        })
        .collect::<TokenStream>();
    let contract_instance = match ep.is_mut {
        true => quote!(let mut contract = #contract_path::instance("contract");),
        false => quote!(let contract = #contract_path::instance("contract");)
    };
    quote! {
        stringify!(#fn_ident) => {
            #contract_instance
            #get_args
            #add_attrs
            #action_attr

            contract.#fn_ident(#args);
        }
    }
}

fn get_args(ep: &Entrypoint) -> TokenStream {
    ep.args
        .iter()
        .enumerate()
        .map(|(idx, arg)| {
            let ident = format_ident!("{}", arg.ident);
            quote!(let #ident = get_arg(action.args.get(#idx).unwrap_or_revert().clone());)
        })
        .collect::<TokenStream>()
}

fn add_attributes(ep: &Entrypoint) -> TokenStream {
    ep.args
        .iter()
        .map(|arg| {
            let ident = format_ident!("{}", arg.ident);
            quote!(odra::cosmos::add_attribute(stringify!(#ident), #ident);)
        })
        .collect::<TokenStream>()
}

fn add_action_attribute(ep: &Entrypoint) -> TokenStream {
    let ident = format_ident!("{}", ep.ident);
    quote!(odra::cosmos::add_attribute("action", stringify!(#ident));)
}

pub fn build_variant_matching<'a, F, M>(
    def: &ContractDef,
    contract_path: &Path,
    f: F,
    mut m: M,
    add_action_attr: bool
) -> TokenStream
where
    F: FnMut(&&Entrypoint) -> bool,
    M: FnMut(&Entrypoint, &Path, bool) -> TokenStream
{
    def.entrypoints
        .iter()
        .filter(f)
        .map(|ep| m(ep, contract_path, add_action_attr))
        .collect::<TokenStream>()
}

pub fn parse_message() -> TokenStream {
    quote! {
        match odra::cosmos::cosmwasm_std::from_slice(input) {
            Ok(val) => val,
            Err(err) => {
                return Err(err.to_string())
            }
        };
    }
}

pub mod query {
    use odra_types::contract_def::Entrypoint;
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use syn::Path;

    use super::get_args;

    pub fn parse_message() -> TokenStream {
        quote! {
            match odra::cosmos::cosmwasm_std::from_slice(input) {
                Ok(val) => val,
                Err(err) => {
                    return Err(err)
                }
            };
        }
    }

    pub fn to_variant_branch(
        ep: &Entrypoint,
        contract_path: &Path,
        _add_action_attr: bool
    ) -> TokenStream {
        let fn_ident = format_ident!("{}", ep.ident);
        let get_args = get_args(ep);
        let args = ep
            .args
            .iter()
            .map(|arg| {
                let ident = format_ident!("{}", arg.ident);
                quote!(#ident,)
            })
            .collect::<TokenStream>();
        let contract_instance = match ep.is_mut {
            true => quote!(let mut contract = #contract_path::instance("contract");),
            false => quote!(let contract = #contract_path::instance("contract");)
        };
        quote! {
            stringify!(#fn_ident) => {
                #contract_instance
                #get_args
                let result = contract.#fn_ident(#args);
                odra::cosmos::cosmwasm_std::to_binary(&result)
            }
        }
    }
}
