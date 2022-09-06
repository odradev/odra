use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

pub(crate) fn generate_fn_body<T>(
    args: T,
    entrypoint_name: &String,
    ret: &syn::ReturnType,
) -> TokenStream
where
    T: IntoIterator<Item = syn::PatType>,
{
    let args = parse_args(args);

    match ret {
        syn::ReturnType::Default => quote! {
            #args
            odra_env::call_contract::<()>(&self.address, #entrypoint_name, &args);
        },
        syn::ReturnType::Type(_, _) => quote! {
            use odra_types::CLTyped;
            #args
            odra_env::call_contract(&self.address, #entrypoint_name, &args)
        },
    }
}

pub(crate) fn filter_args<'a, T>(args: T) -> Vec<syn::PatType>
where
    T: IntoIterator<Item = &'a syn::FnArg>,
{
    args.into_iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(pat) => Some(pat.clone()),
        })
        .collect::<Vec<_>>()
}

fn parse_args<T>(syn_args: T) -> TokenStream
where
    T: IntoIterator<Item = syn::PatType>,
{
    let mut tokens = quote!(let mut args = RuntimeArgs::new(););
    tokens.append_all(syn_args.into_iter().map(|arg| {
        let pat = &*arg.pat;
        quote! { args.insert(stringify!(#pat), #pat).unwrap(); }
    }));
    tokens.extend(quote!(args));

    quote! {
        use odra_types::RuntimeArgs;
        let args = {
            #tokens
        };
    }
}
