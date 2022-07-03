use odra_ir::ExternalContractItem;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};

//TODO: should share some common code with reference.rs
pub fn generate_code(item: ExternalContractItem) -> TokenStream {
    let item_trait = item.item_trait();
    let trait_ident = &item_trait.ident;
    let ref_ident = format_ident!("{}Ref", &item_trait.ident);

    let methods = item_trait
        .items
        .iter()
        .filter_map(|item| match item {
            syn::TraitItem::Method(method) => Some(method),
            _ => None,
        })
        .map(|item| {
            let sig = &item.sig;
            let entrypoint_name = &item.sig.ident.to_string();
            let args = &sig
                .inputs
                .iter()
                .filter_map(|arg| match arg {
                    syn::FnArg::Receiver(_) => None,
                    syn::FnArg::Typed(pat) => Some(pat.clone()),
                })
                .collect::<Vec<_>>();
            let ret = &sig.output;

            let fn_body = generate_fn_body(entrypoint_name, args, ret);
            quote::quote! {
                #sig {
                    #fn_body
                }
            }
        })
        .flatten()
        .collect::<TokenStream>();
    quote::quote! {
        #item_trait

        pub struct #ref_ident {
            address: odra::types::Address,
        }

        impl #ref_ident {
            fn at(address: odra::types::Address) -> Self {
                Self { address }
            }
        }

        impl #trait_ident for #ref_ident {
            #methods
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
