use std::convert::TryFrom;

use crate::{
    attrs::{partition_attributes, OdraAttribute},
    module_item::utils,
};
use quote::{quote, ToTokens};

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
    pub full_sig: syn::Signature,
}

impl ToTokens for Constructor {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.ident.to_string();
        let args = &self
            .args
            .iter()
            .flat_map(|arg| {
                let name = &*arg.pat;
                let ty = &*arg.ty;
                let ty = quote!(<#ty as odra::types::CLTyped>::cl_type());
                quote! {
                    odra::contract_def::Argument {
                        ident: String::from(stringify!(#name)),
                        ty: #ty,
                    },
                }
            })
            .collect::<proc_macro2::TokenStream>();
        let ep = quote! {
            odra::contract_def::Entrypoint {
                ident: String::from(#name),
                args: vec![#args],
                ret: odra::types::CLType::Unit,
                ty: odra::contract_def::EntrypointType::Constructor,
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
        let args = method
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(pat) => Some(pat.clone()),
            })
            .collect::<syn::punctuated::Punctuated<syn::PatType, syn::token::Comma>>();
        if let syn::ReturnType::Type(_, _) = method.sig.output {
            return Err(syn::Error::new_spanned(
                method.sig,
                "Constructor must not return value.",
            ));
        }
        let full_sig = method.clone().sig;

        let is_payable = odra_attrs.iter().any(|attr| attr.is_payable());
        let block = utils::payable_check(method.block, is_payable);

        Ok(Self {
            attrs: odra_attrs,
            impl_item: syn::ImplItemMethod {
                attrs,
                block,
                ..method
            },
            ident,
            args,
            full_sig,
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
            #[odra(init)]
            #[some(a)]
            pub fn set_initial_value(&self, value: u32) {
                self.set_value(value);
            }
        };
        let constructor = Constructor::try_from(item).unwrap();
        assert_eq!(constructor.attrs.len(), 1);
    }
}
