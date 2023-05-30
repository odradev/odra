use odra_types::{
    contract_def::{Entrypoint, EntrypointType},
    Type
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{self, punctuated::Punctuated, token::Comma, Path};

use super::arg::CasperArgs;

pub(crate) struct WasmEntrypoint<'a>(pub &'a Entrypoint, pub &'a Path);

impl ToTokens for WasmEntrypoint<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let entrypoint_ident = format_ident!("{}", &self.0.ident);
        let args = CasperArgs(&self.0.args).to_token_stream();

        let mut fn_args = Punctuated::<TokenStream, Comma>::new();
        self.0
            .args
            .iter()
            .map(|arg| {
                let ident = format_ident!("{}", arg.ident);
                if arg.is_ref {
                    quote!(&#ident)
                } else {
                    quote!(#ident)
                }
            })
            .for_each(|stream: TokenStream| fn_args.push(stream));

        let payable = match self.0.ty {
            EntrypointType::PublicPayable { .. } => quote! {
                odra::casper::utils::handle_attached_value();
            },
            _ => quote! {
                odra::casper::utils::assert_no_attached_value();
            }
        };

        let payable_cleanup = match self.0.ty {
            EntrypointType::PublicPayable { .. } => quote! {
                odra::casper::utils::clear_attached_value();
            },
            _ => quote!()
        };

        let non_reentrant_before = if self.0.ty.is_non_reentrant() {
            quote!(odra::casper::utils::non_reentrant_before();)
        } else {
            quote!()
        };

        let non_reentrant_after = if self.0.ty.is_non_reentrant() {
            quote!(odra::casper::utils::non_reentrant_after();)
        } else {
            quote!()
        };

        let contract_call = match self.0.ret {
            Type::Unit => quote! {
                #args
                contract.#entrypoint_ident(#fn_args);
            },
            _ => quote! {
                use odra::casper::casper_contract::unwrap_or_revert::UnwrapOrRevert;
                #args
                let result = contract.#entrypoint_ident(#fn_args);
                let result = odra::casper::casper_types::CLValue::from_t(result).unwrap_or_revert();
            }
        };

        let return_stmt = match self.0.ret {
            Type::Unit => quote!(),
            _ => quote!(odra::casper::casper_contract::contract_api::runtime::ret(result);)
        };

        let contract_path = &self.1;
        let contract_instance = match self.0.is_mut {
            true => quote!(let mut contract = #contract_path::instance("contract");),
            false => quote!(let contract = #contract_path::instance("contract");)
        };

        tokens.extend(quote! {
            #[no_mangle]
            fn #entrypoint_ident() {
                #non_reentrant_before
                #payable
                #contract_instance
                #contract_call
                #payable_cleanup
                #non_reentrant_after
                #return_stmt
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_eq_tokens;
    use odra_types::contract_def::{Argument, EntrypointType};

    use super::*;

    #[test]
    fn test_constructor() {
        let entrypoint = Entrypoint {
            ident: String::from("construct_me"),
            args: vec![Argument {
                ident: String::from("value"),
                ty: Type::I32,
                is_ref: true
            }],
            ret: Type::Unit,
            ty: EntrypointType::Public {
                non_reentrant: false
            },
            is_mut: false
        };
        let path: Path = syn::parse2(
            quote! {
                my_contract::MyContract
            }
            .to_token_stream()
        )
        .unwrap();

        let wasm_entrypoint = WasmEntrypoint(&entrypoint, &path);
        assert_eq_tokens(
            wasm_entrypoint,
            quote!(
                #[no_mangle]
                fn construct_me() {
                    odra::casper::utils::assert_no_attached_value();
                    let contract = my_contract::MyContract::instance("contract");
                    let value = odra::casper::casper_contract::contract_api::runtime::get_named_arg(
                        &stringify!(value)
                    );
                    contract.construct_me(value);
                }
            )
        );
    }

    #[test]
    fn test_payable() {
        let entrypoint = Entrypoint {
            ident: String::from("pay_me"),
            args: vec![],
            ret: Type::Unit,
            ty: EntrypointType::PublicPayable {
                non_reentrant: false
            },
            is_mut: false
        };
        let path: Path = syn::parse_quote!(a::b::c::Contract);

        let wasm_entrypoint = WasmEntrypoint(&entrypoint, &path);
        assert_eq_tokens(
            wasm_entrypoint,
            quote!(
                #[no_mangle]
                fn pay_me() {
                    odra::casper::utils::handle_attached_value();
                    let contract = a::b::c::Contract::instance("contract");
                    contract.pay_me();
                    odra::casper::utils::clear_attached_value();
                }
            )
        );
    }

    #[test]
    fn test_mutable() {
        let entrypoint = Entrypoint {
            ident: String::from("pay_me"),
            args: vec![],
            ret: Type::Unit,
            ty: EntrypointType::PublicPayable {
                non_reentrant: false
            },
            is_mut: true
        };
        let path: Path = syn::parse_quote!(a::b::c::Contract);

        let wasm_entrypoint = WasmEntrypoint(&entrypoint, &path);
        assert_eq_tokens(
            wasm_entrypoint,
            quote!(
                #[no_mangle]
                fn pay_me() {
                    odra::casper::utils::handle_attached_value();
                    let mut contract = a::b::c::Contract::instance("contract");
                    contract.pay_me();
                    odra::casper::utils::clear_attached_value();
                }
            )
        );
    }
}
