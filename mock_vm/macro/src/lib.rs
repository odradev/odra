extern crate proc_macro;

use proc_macro::TokenStream;
use quote::format_ident;
use syn::{punctuated::Punctuated, spanned::Spanned, ItemFn};

#[proc_macro_attribute]
pub fn event_test(_: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = syn::parse2::<syn::ItemFn>(item.clone().into()).unwrap();

    if let Err(error) = validity_checks(&item_fn) {
        return error.to_compile_error().into();
    };

    let sig = &item_fn.sig;
    let events_arg: syn::FnArg = syn::parse_quote!(events: Vec<EventData>);
    let address_arg: syn::FnArg = syn::parse_quote!(contract_address: &Address);

    let mut inputs = sig
        .inputs
        .clone()
        .into_iter()
        .filter(|input| &address_arg != input)
        .collect::<Punctuated<_, _>>();
    inputs.insert(0, events_arg);

    let new_sig = syn::Signature {
        inputs: inputs.clone(),
        ident: format_ident!("test_{}", sig.ident.to_string()),
        ..sig.clone()
    };

    let original_fn: proc_macro2::TokenStream = item.into();

    let stmts = item_fn.block.stmts.into_iter().skip(1);

    let res: proc_macro2::TokenStream = quote::quote! {
        #original_fn

        #new_sig {
            # ( #stmts )*
        }
    };

    res.into()
}

fn validity_checks(item_fn: &ItemFn) -> Result<(), syn::Error> {
    let address_arg: syn::FnArg = syn::parse_quote!(contract_address: &Address);
    let first_arg = item_fn.sig.inputs.first().unwrap();

    if first_arg != &address_arg {
        return Err(syn::Error::new(
            first_arg.span(),
            "First arg must be exactly `contract_address: &Address`",
        ));
    }

    let events_assignment: syn::Stmt = syn::parse_quote!(let events = events(contract_address););
    let first_stmt = item_fn.block.stmts.first().unwrap();

    if &events_assignment != first_stmt {
        return Err(syn::Error::new(
            first_stmt.span(),
            "First statement must be `let events = events(contract_address);`",
        ));
    }
    Ok(())
}
