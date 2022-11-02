use odra_types::contract_def::Entrypoint;
use quote::{format_ident, quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, Ident, Path};

use super::arg::CasperArgs;
type FnArgs = Punctuated<Ident, Comma>;

pub struct WasmConstructor<'a>(pub Vec<&'a Entrypoint>, pub &'a Path);

impl ToTokens for WasmConstructor<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let data: Vec<(Ident, CasperArgs, FnArgs)> = self
            .0
            .iter()
            .map(|ep| {
                let entrypoint_ident = format_ident!("{}", &ep.ident);
                let casper_args = CasperArgs(&ep.args);

                let mut fn_args = Punctuated::<Ident, Comma>::new();
                ep.args
                    .iter()
                    .for_each(|arg| fn_args.push(format_ident!("{}", arg.ident)));

                (entrypoint_ident, casper_args, fn_args)
            })
            .collect();

        let ref_ident = &self.1;
        let constructor_matching: proc_macro2::TokenStream = data
            .iter()
            .flat_map(|(entrypoint_ident, casper_args, fn_args)| {
                quote! {
                    stringify!(#entrypoint_ident) => {
                        let odra_address = odra::types::Address::from(contract_package_hash);
                        let contract_ref = #ref_ident::at(odra_address);
                        #casper_args
                        contract_ref.#entrypoint_ident( #fn_args );
                    },
                }
            })
            .collect();

        tokens.extend(quote! {
            if odra::casper::utils::named_arg_exists("constructor") {
                use odra::casper::casper_contract::unwrap_or_revert::UnwrapOrRevert;
                let constructor_access: odra::casper::casper_types::URef =
                    odra::casper::casper_contract::contract_api::storage::create_contract_user_group(
                        contract_package_hash,
                        "constructor",
                        1,
                        Default::default()
                    )
                    .unwrap_or_revert()
                    .pop()
                    .unwrap_or_revert();

                let constructor_name = odra::casper::casper_contract::contract_api::runtime::get_named_arg::<String>(
                    "constructor"
                );
                let constructor_name = constructor_name.as_str();

                match constructor_name {
                    #constructor_matching
                    _ => {}
                };

                // Revoke access to constructor.
                let mut urefs = std::collections::BTreeSet::new();
                urefs.insert(constructor_access);
                odra::casper::casper_contract::contract_api::storage::remove_contract_user_group_urefs(
                    contract_package_hash,
                    "constructor",
                    urefs
                ).unwrap_or_revert();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use odra_types::{
        contract_def::{Argument, EntrypointType},
        Type,
    };

    use crate::assert_eq_tokens;

    use super::*;

    #[test]
    fn test_constructor() {
        let constructor = Entrypoint {
            ident: String::from("construct_me"),
            args: vec![Argument {
                ident: String::from("value"),
                ty: Type::I32,
            }],
            ret: Type::Unit,
            ty: EntrypointType::Constructor,
        };
        let path: Path = syn::parse2(
            quote! {
                my_contract::MyContract
            }
            .to_token_stream(),
        )
        .unwrap();

        let wasm_constructor = WasmConstructor(vec![&constructor], &path);
        assert_eq_tokens(
            wasm_constructor,
            quote! {
                if odra::casper::utils::named_arg_exists("constructor") {
                    use odra::casper::casper_contract::unwrap_or_revert::UnwrapOrRevert;
                    let constructor_access: odra::casper::casper_types::URef = odra::casper::casper_contract::contract_api::storage::create_contract_user_group(
                        contract_package_hash , "constructor" , 1 , Default::default()
                    ).unwrap_or_revert().pop().unwrap_or_revert();
                    let constructor_name = odra::casper::casper_contract::contract_api::runtime::get_named_arg::<String>("constructor");
                    let constructor_name = constructor_name.as_str();
                    match constructor_name {
                        stringify!(construct_me) => {
                            let odra_address = odra::types::Address::from(contract_package_hash);
                            let contract_ref = my_contract::MyContract::at(odra_address);
                            let value = odra::casper::casper_contract::contract_api::runtime::get_named_arg (stringify!(value));
                            contract_ref.construct_me(value);
                        },
                        _ => {}
                    };
                    let mut urefs = std::collections::BTreeSet::new();
                    urefs.insert(constructor_access);
                    odra::casper::casper_contract::contract_api::storage::remove_contract_user_group_urefs(
                        contract_package_hash , "constructor" , urefs
                    ).unwrap_or_revert();
                }
            },
        );
    }
}
