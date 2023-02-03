use proc_macro2::{Ident, TokenStream};
use quote::{quote, TokenStreamExt};

pub(crate) fn generate_fn_body<T>(
    args: T,
    entrypoint_name: &String,
    ret: &syn::ReturnType
) -> TokenStream
where
    T: IntoIterator<Item = syn::PatType>
{
    let args = parse_args(args);

    match ret {
        syn::ReturnType::Default => quote! {
            #args
            odra::call_contract::<()>(self.address, #entrypoint_name, args, self.attached_value);
        },
        syn::ReturnType::Type(_, _) => quote! {
            #args
            odra::call_contract(self.address, #entrypoint_name, args, self.attached_value)
        }
    }
}

pub(crate) fn filter_args<'a, T>(args: T) -> Vec<syn::PatType>
where
    T: IntoIterator<Item = &'a syn::FnArg>
{
    args.into_iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(pat) => Some(pat.clone())
        })
        .collect::<Vec<_>>()
}

pub(crate) fn build_ref(ref_ident: &Ident) -> TokenStream {
    quote! {
        pub struct #ref_ident {
            address: odra::types::Address,
            attached_value: Option<odra::types::Balance>,
        }

        impl #ref_ident {
            pub fn at(address: odra::types::Address) -> Self {
                Self { address, attached_value: None }
            }

            pub fn address(&self) -> odra::types::Address {
                self.address.clone()
            }

            pub fn with_tokens<T>(&self, amount: T) -> Self
            where T: Into<odra::types::Balance> {
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
    T: IntoIterator<Item = syn::PatType>
{
    let mut tokens = quote!(let mut args = odra::types::CallArgs::new(););
    tokens.append_all(syn_args.into_iter().map(|arg| {
        let pat = &*arg.pat;
        quote! { args.insert(stringify!(#pat), #pat); }
    }));
    tokens.extend(quote!(args));

    quote! {
        let args = {
            #tokens
        };
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use proc_macro2::TokenStream;
    use quote::{quote, ToTokens};
    use syn::{parse, parse2, parse_quote, Type};

    #[test]
    fn ddd() {
        let ty_str = "odra :: types :: Address";

        let ts = TokenStream::from_str(stringify!(u32)).unwrap();
        let ts = TokenStream::from_str(ty_str).unwrap();

        let ty = parse2::<Type>(ts);
        // dbg!(ty);
        // let ty: Type = parse_quote!(u32);
        // dbg!(ty);
        // let s = ty.unwrap().to_token_stream();
        // dbg!(s.to_string());

        let ty = ty.unwrap();
        // let ty = ty.to_token_stream();

        let a = quote! {
            odra::types::contract_def::Argument {
                ty: String::from(stringify!(#ty)),
            },
        };

        dbg!(a.to_string());
    }
}
