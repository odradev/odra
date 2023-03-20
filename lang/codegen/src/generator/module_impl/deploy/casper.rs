use odra_ir::module::{Constructor, ImplItem, ModuleImpl};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, ReturnType, Type, TypePath};

use crate::generator::module_impl::deploy::args_to_runtime_args_stream;

pub fn build_constructors(contract: &ModuleImpl) -> TokenStream {
    let struct_ident = contract.ident();
    let struct_name = struct_ident.to_string();
    let ref_ident = format_ident!("{}Ref", struct_ident);
    let struct_snake_case = odra_utils::camel_to_snake(&struct_name);

    let mut constructors_wasm_test = build_constructors_wasm_test(
        contract
            .methods()
            .iter()
            .filter_map(|item| match item {
                ImplItem::Constructor(constructor) => Some(constructor),
                _ => None
            }),
        struct_ident,
        ref_ident.clone()
    );
    if constructors_wasm_test.is_empty() {
        constructors_wasm_test = quote! {
            pub fn default() -> #ref_ident {
                let address = odra::test_env::register_contract(&#struct_snake_case, odra::types::CallArgs::new());
                #ref_ident::at(address)
            }
        };
    }
    constructors_wasm_test
}

fn build_constructors_wasm_test<'a, C>(
    constructors: C,
    struct_ident: &Ident,
    ref_ident: Ident
) -> TokenStream
where
    C: Iterator<Item = &'a Constructor>
{
    let struct_name = struct_ident.to_string();
    let struct_name_snake_case = odra_utils::camel_to_snake(&struct_name);

    constructors
        .map(|constructor| {
            let ty = Type::Path(TypePath {
                qself: None,
                path: From::from(ref_ident.clone())
            });
            let sig = constructor.full_sig.clone();
            let constructor_ident = &constructor.ident;

            let inputs = sig
                .inputs
                .into_iter()
                .filter(|i| match i {
                    syn::FnArg::Receiver(_) => false,
                    syn::FnArg::Typed(_) => true
                })
                .collect::<Punctuated<_, _>>();

            let fn_sig = syn::Signature {
                output: ReturnType::Type(Default::default(), Box::new(ty)),
                inputs,
                ..sig
            };

            let args = args_to_runtime_args_stream(&constructor.args);

            quote! {
                pub #fn_sig {
                    let mut args = { #args };
                    args.insert("constructor", stringify!(#constructor_ident));
                    let address = odra::test_env::register_contract(#struct_name_snake_case, args);
                    #ref_ident::at(address)
                }
            }
        })
        .collect::<TokenStream>()
}