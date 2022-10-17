use odra::contract_def::Entrypoint;
use quote::{format_ident, quote, ToTokens};
use syn::{self, punctuated::Punctuated, token::Comma, Ident, Path};

use super::arg::CasperArgs;

pub(crate) struct WasmEntrypoint<'a>(pub &'a Entrypoint, pub &'a Path);

impl ToTokens for WasmEntrypoint<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let entrypoint_ident = format_ident!("{}", &self.0.ident);
        let args = CasperArgs(&self.0.args).to_token_stream();

        let mut fn_args = Punctuated::<Ident, Comma>::new();
        self.0
            .args
            .iter()
            .for_each(|arg| fn_args.push(format_ident!("{}", arg.ident)));

        let payable = match self.0.ty {
            odra::contract_def::EntrypointType::PublicPayable => quote! {
                casper_backend::backend::handle_attached_value();
            },
            _ => quote! {
                casper_backend::backend::assert_no_attached_value();
            },
        };

        let payable_cleanup = match self.0.ty {
            odra::contract_def::EntrypointType::PublicPayable => quote! {
                casper_backend::backend::clear_attached_value();
            },
            _ => quote!(),
        };

        let contract_call = match self.0.ret {
            odra::types::CLType::Unit => quote! {
                #args
                contract.#entrypoint_ident(#fn_args);
            },
            _ => quote! {
                use casper_backend::backend::casper_contract::unwrap_or_revert::UnwrapOrRevert;
                #args
                let result = contract.#entrypoint_ident(#fn_args);
                let result = casper_backend::backend::casper_types::CLValue::from_t(result).unwrap_or_revert();
                casper_backend::backend::casper_contract::contract_api::runtime::ret(result);
            },
        };

        let contract_path = &self.1;

        tokens.extend(quote! {
            #[no_mangle]
            fn #entrypoint_ident() {
                #payable
                let contract = #contract_path::instance("contract");
                #contract_call
                #payable_cleanup
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::assert_eq_tokens;
    use odra::contract_def::{Argument, EntrypointType};
    use odra::types::CLType;

    use super::*;

    #[test]
    fn test_constructor() {
        let entrypoint = Entrypoint {
            ident: String::from("construct_me"),
            args: vec![Argument {
                ident: String::from("value"),
                ty: CLType::I32,
            }],
            ret: CLType::Unit,
            ty: EntrypointType::Public,
        };
        let path: Path = syn::parse2(
            quote! {
                my_contract::MyContract
            }
            .to_token_stream(),
        )
        .unwrap();

        let wasm_entrypoint = WasmEntrypoint(&entrypoint, &path);
        assert_eq_tokens(
            wasm_entrypoint,
            quote!(
                #[no_mangle]
                fn construct_me() {
                    let contract = my_contract::MyContract::instance("contract");
                    let value = casper_backend::backend::casper_contract::contract_api::runtime::get_named_arg(stringify!(value));
                    contract.construct_me(value);
                }
            ),
        );
    }

    #[test]
    fn test_payable() {
        let entrypoint = Entrypoint {
            ident: String::from("pay_me"),
            args: vec![],
            ret: CLType::Unit,
            ty: EntrypointType::PublicPayable,
        };
        let path: Path = syn::parse_quote!(a::b::c::Contract);

        let wasm_entrypoint = WasmEntrypoint(&entrypoint, &path);
        assert_eq_tokens(
            wasm_entrypoint,
            quote!(
                #[no_mangle]
                fn pay_me() {
                    let cargo_purse = casper_backend::backend::casper_contract::contract_api::runtime::get_named_arg("purse");
                    let amount = casper_backend::backend::casper_contract::contract_api::system::get_purse_balance(cargo_purse).unwrap_or_default();
                    if amount > odra::types::U512::zero() {
                        let contract_purse = casper_backend::backend::get_main_purse();
                        casper_backend::backend::casper_contract::contract_api::system::transfer_from_purse_to_purse(cargo_purse, contract_purse, amount, None);
                        casper_backend::backend::set_attached_value(amount);
                    }
                    let contract = a::b::c::Contract::instance("contract");
                    contract.pay_me();
                    casper_backend::backend::clear_attached_value();
                }
            ),
        );
    }
}
