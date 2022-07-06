use derive_more::From;
use odra_ir::contract_item::{contract_impl::ContractImpl, impl_item::ImplItem};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};
use syn::token::RArrow;

use crate::GenerateCode;

#[derive(From)]
pub struct ContractReference<'a> {
    contract: &'a ContractImpl,
}

as_ref_for_contract_impl_generator!(ContractReference);

impl GenerateCode for ContractReference<'_> {
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.contract.ident();
        let ref_ident = format_ident!("{}Ref", struct_ident);

        let methods = self.contract.methods();

        let ref_entrypoints = build_entrypoints(&methods);

        let ref_constructors = build_constructors(&methods);

        quote! {
            pub struct #ref_ident {
                address: odra::types::Address,
            }

            impl #ref_ident {
                #ref_entrypoints

                #ref_constructors

                pub fn address(&self) -> odra::types::Address {
                    self.address.clone()
                }

                pub fn at(address: odra::types::Address) -> Self {
                    Self { address }
                }
            }
        }
    }
}

fn parse_args(syn_args: &Vec<syn::PatType>) -> TokenStream {
    let args = match &syn_args.is_empty() {
        true => quote! { RuntimeArgs::new()},
        false => {
            let mut args = quote!(let mut args = RuntimeArgs::new(););
            args.append_all(syn_args.clone().into_iter().map(|arg| {
                let pat = &*arg.pat;
                quote! { args.insert(stringify!(#pat), #pat).unwrap(); }
            }));
            args.extend(quote!(args));
            args
        }
    };

    quote! {
        use odra::types::RuntimeArgs;
        let args = {
            #args
        };
    }
}

fn build_entrypoints(methods: &Vec<&ImplItem>) -> TokenStream {
    methods
        .iter()
        .filter_map(|item| match item {
            ImplItem::Method(method) => Some(method),
            _ => None,
        })
        .map(|entrypoint| {
            let sig = &entrypoint.full_sig;
            let entrypoint_name = &entrypoint.ident.to_string();
            let fn_body = generate_fn_body(entrypoint_name, &entrypoint.args, &entrypoint.ret);

            let test_sig = sig_to_test_sig(sig);

            quote! {
                pub #sig {
                    #fn_body
                }
            }
        })
        .collect::<TokenStream>()
}

fn build_constructors(methods: &Vec<&ImplItem>) -> TokenStream {
    methods
        .iter()
        .filter_map(|item| match item {
            ImplItem::Constructor(constructor) => Some(constructor),
            _ => None,
        })
        .map(|entrypoint| {
            let sig = &entrypoint.full_sig;
            let entrypoint_name = &entrypoint.ident.to_string();
            let fn_body =
                generate_fn_body(entrypoint_name, &entrypoint.args, &syn::ReturnType::Default);
            
            let test_sig = sig_to_test_sig(sig);

            quote! {
                pub #sig {
                    #fn_body
                }
            }
        })
        .collect::<TokenStream>()
}

fn generate_fn_body(
    entrypoint_name: &String,
    args: &Vec<syn::PatType>,
    ret: &syn::ReturnType,
) -> TokenStream {
    let args = parse_args(args);

    match ret {
        syn::ReturnType::Default => quote! {
            #args
            odra::call_contract::<()>(&self.address, #entrypoint_name, &args);
        },
        syn::ReturnType::Type(_, _) => quote! {
            use odra::types::CLTyped;
            #args
            odra::call_contract(&self.address, #entrypoint_name, &args)
        },
    }
}

fn sig_to_test_sig(sig: &syn::Signature) -> syn::Signature {
    let ty: syn::Type = match &sig.output {
        syn::ReturnType::Default => syn::parse_quote!(Result<(), odra::types::OdraError>),
        syn::ReturnType::Type(_, ty) => {
            let ty = &**ty;
            syn::parse_quote!(Result<#ty, odra::types::OdraError>)
        },
    };

    syn::Signature { 
        output: syn::ReturnType::Type(RArrow::default(), Box::new(ty)),
        ..sig.to_owned()
    }
}