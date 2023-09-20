use quote::{quote, ToTokens};

use crate::attrs::{partition_attributes, OdraAttribute};

use super::utils;

/// Odra method definition.
///
/// # Examples
/// ```
/// # <odra_ir::module::Method as TryFrom<syn::ImplItemMethod>>::try_from(syn::parse_quote! {
/// pub fn set_value(&self, value: u32) {
///    // logic goes here
/// }
/// # });
/// ```
pub struct Method {
    pub attrs: Vec<OdraAttribute>,
    pub impl_item: syn::ImplItemMethod,
    pub ident: syn::Ident,
    pub is_mut: bool,
    pub args: syn::punctuated::Punctuated<syn::PatType, syn::token::Comma>,
    pub ret: syn::ReturnType,
    pub full_sig: syn::Signature,
    pub visibility: syn::Visibility
}

impl Method {
    pub fn is_public(&self) -> bool {
        matches!(self.visibility, syn::Visibility::Public(_))
    }

    pub fn is_payable(&self) -> bool {
        self.attrs.iter().any(|attr| attr.is_payable())
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let is_non_reentrant = self.attrs.iter().any(OdraAttribute::is_non_reentrant);
        let name = &self.ident.to_string();
        let args = &self
            .args
            .iter()
            .map(|arg| {
                let name = &*arg.pat;

                let (ty, is_slice) = utils::ty(arg);
                let is_ref = utils::is_ref(arg);
                let ty = quote!(<#ty as odra::types::CLTyped>::cl_type());
                quote! {
                    odra::types::contract_def::Argument {
                        ident: odra::prelude::string::String::from(stringify!(#name)),
                        ty: #ty,
                        is_ref: #is_ref,
                        is_slice: #is_slice
                    },
                }
            })
            .collect::<proc_macro2::TokenStream>();

        let ret = match &self.ret {
            syn::ReturnType::Default => quote!(odra::types::CLType::Unit),
            syn::ReturnType::Type(_, ty) => {
                quote!(<#ty as odra::types::CLTyped>::cl_type())
            }
        };

        let ty = match self.attrs.iter().any(|attr| attr.is_payable()) {
            true => {
                quote!(odra::types::contract_def::EntrypointType::PublicPayable { non_reentrant: #is_non_reentrant })
            }
            false => {
                quote!(odra::types::contract_def::EntrypointType::Public { non_reentrant: #is_non_reentrant })
            }
        };

        let is_mut = self.is_mut;

        let ep = quote! {
            odra::types::contract_def::Entrypoint {
                ident: odra::prelude::string::String::from(#name),
                args: odra::prelude::vec![#args],
                is_mut: #is_mut,
                ret: #ret,
                ty: #ty,
            },
        };

        tokens.extend(ep)
    }
}

impl TryFrom<syn::ImplItemMethod> for Method {
    type Error = syn::Error;

    fn try_from(method: syn::ImplItemMethod) -> Result<Self, Self::Error> {
        let (odra_attrs, attrs) = partition_attributes(method.clone().attrs)?;
        let ident = method.sig.ident.to_owned();
        let args = utils::extract_typed_inputs(&method.sig);
        let ret = method.clone().sig.output;
        let full_sig = method.clone().sig;
        let visibility = method.vis.clone();
        let is_mut = utils::is_mut(&full_sig);

        Ok(Self {
            attrs: odra_attrs,
            impl_item: syn::ImplItemMethod { attrs, ..method },
            ident,
            is_mut,
            args,
            ret,
            full_sig,
            visibility
        })
    }
}
