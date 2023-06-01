use std::convert::TryFrom;

use crate::attrs::{partition_attributes, OdraAttribute};
use quote::{quote, ToTokens};

use super::utils;

/// Odra constructor definition.
///
/// # Examples
/// ```
/// # <odra_ir::module::Constructor as TryFrom<syn::ImplItemMethod>>::try_from(syn::parse_quote! {
/// #[odra(init)]
/// #[other_attribute]
/// pub fn set_initial_value(&self, value: u32) {
///     // initialization logic goes here
/// }
/// # }).unwrap();
/// ```
pub struct Constructor {
    pub attrs: Vec<OdraAttribute>,
    pub impl_item: syn::ImplItemMethod,
    pub ident: syn::Ident,
    pub args: syn::punctuated::Punctuated<syn::PatType, syn::token::Comma>,
    pub full_sig: syn::Signature
}

impl ToTokens for Constructor {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let is_non_reentrant = self.attrs.iter().any(OdraAttribute::is_non_reentrant);
        let is_mut = utils::is_mut(&self.full_sig);
        let name = &self.ident.to_string();
        let args = &self
            .args
            .iter()
            .flat_map(|arg| {
                let name = &*arg.pat;
                let ty = utils::ty(arg);
                let is_ref = utils::is_ref(arg);
                let ty = quote!(<#ty as odra::types::Typed>::ty());
                quote! {
                    odra::types::contract_def::Argument {
                        ident: String::from(stringify!(#name)),
                        ty: #ty,
                        is_ref: #is_ref,
                    },
                }
            })
            .collect::<proc_macro2::TokenStream>();
        let ep = quote! {
            odra::types::contract_def::Entrypoint {
                ident: String::from(#name),
                args: vec![#args],
                is_mut: #is_mut,
                ret: odra::types::Type::Unit,
                ty: odra::types::contract_def::EntrypointType::Constructor { non_reentrant: #is_non_reentrant },
            },
        };

        tokens.extend(ep)
    }
}

impl TryFrom<syn::ImplItemMethod> for Constructor {
    type Error = syn::Error;

    fn try_from(method: syn::ImplItemMethod) -> Result<Self, Self::Error> {
        let (odra_attrs, attrs) = partition_attributes(method.clone().attrs).unwrap();
        let ident = method.sig.ident.to_owned();
        let args = utils::extract_typed_inputs(&method.sig);
        if let syn::ReturnType::Type(_, _) = method.sig.output {
            return Err(syn::Error::new_spanned(
                method.sig,
                "Constructor must not return value."
            ));
        }
        let full_sig = method.sig.clone();

        Ok(Self {
            attrs: odra_attrs,
            impl_item: syn::ImplItemMethod { attrs, ..method },
            ident,
            args,
            full_sig
        })
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use super::Constructor;

    #[test]
    fn test_attrs() {
        let item: syn::ImplItemMethod = syn::parse_quote! {
            #[odra(init, non_reentrant)]
            #[some(a)]
            pub fn set_initial_value(&self, value: u32) {
                self.set_value(value);
            }
        };
        let constructor = Constructor::try_from(item).unwrap();
        assert_eq!(constructor.attrs.len(), 1);
    }
}
