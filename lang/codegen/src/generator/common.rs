use proc_macro2::{TokenStream, Ident};
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
            odra::call_contract::<()>(&self.address, #entrypoint_name, &args, self.attached_value);
        },
        syn::ReturnType::Type(_, _) => quote! {
            use odra::types::CLTyped;
            #args
            odra::call_contract(&self.address, #entrypoint_name, &args, self.attached_value)
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

pub(crate) fn build_ref(ref_ident: &Ident) -> TokenStream {
    quote! {
        pub struct #ref_ident {
            address: odra::types::Address,
            attached_value: Option<odra::types::U512>,
        }

        impl #ref_ident {
            fn at(address: odra::types::Address) -> Self {
                Self { address, attached_value: None }
            }

            fn address(&self) -> odra::types::Address {
                self.address.clone()
            }

            pub fn with_tokens<T>(&self, amount: T) -> Self
            where T: Into<odra::types::U512> {
                Self {
                    address: self.address,
                    attached_value: Some(amount.into()),
                }
            }
        }
    }
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
        use odra::types::RuntimeArgs;
        let args = {
            #tokens
        };
    }
}
