use derive_more::From;
use odra_ir::module::ModuleImpl;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};
use syn::{punctuated::Punctuated, token::Comma};

use crate::GenerateCode;
mod casper;
mod mock_vm;

#[derive(From)]
pub struct Deploy<'a> {
    contract: &'a ModuleImpl
}

as_ref_for_contract_impl_generator!(Deploy);

impl GenerateCode for Deploy<'_> {
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.contract.ident();
        let deployer_ident = format_ident!("{}Deployer", struct_ident);

        let constructors_mock_vm = mock_vm::build_constructors(self.contract);
        let constructors_wasm_test = casper::build_constructors(self.contract);

        quote! {
            pub struct #deployer_ident;

            #[cfg(all(feature = "casper", not(target_arch = "wasm32")))]
            impl #deployer_ident {
                #constructors_wasm_test
            }

            #[cfg(feature = "mock-vm")]
            impl #deployer_ident {
                #constructors_mock_vm
            }
        }
    }
}

fn args_to_fn_args<'a, T>(args: T) -> Punctuated<TokenStream, Comma>
where
    T: IntoIterator<Item = &'a syn::PatType>
{
    args.into_iter()
        .map(|arg| {
            let pat = &*arg.pat;
            quote!(args.get(stringify!(#pat)))
        })
        .collect::<Punctuated<TokenStream, Comma>>()
}

fn args_to_runtime_args_stream<'a, T>(args: T) -> TokenStream
where
    T: IntoIterator<Item = &'a syn::PatType>
{
    let mut tokens = quote!(let mut args = odra::types::CallArgs::new(););
    tokens.append_all(args.into_iter().map(|arg| {
        let pat = &*arg.pat;
        quote! { args.insert(stringify!(#pat), #pat); }
    }));
    tokens.extend(quote!(args));
    tokens
}

fn args_to_arg_names_stream<'a, T>(args: T) -> TokenStream
where
    T: IntoIterator<Item = &'a syn::PatType>
{
    let args_stream = args
        .into_iter()
        .map(|arg| {
            let pat = &*arg.pat;
            quote! { args.push(stringify!(#pat).to_string()); }
        })
        .collect::<TokenStream>();

    quote! {
        {
            let mut args: Vec<String> = vec![];
            #args_stream
            args
        }
    }
}
