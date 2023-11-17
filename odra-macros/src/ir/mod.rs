use crate::syn_utils;
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use syn::{parse_quote, ItemImpl};

pub mod deployer_item;
pub mod host_ref_item;
pub mod ref_item;
pub(crate) mod ref_utils;

pub struct ModuleIR {
    code: ItemImpl
}

impl TryFrom<&TokenStream> for ModuleIR {
    type Error = syn::Error;

    fn try_from(value: &TokenStream) -> Result<Self, Self::Error> {
        Ok(Self {
            code: syn::parse2::<syn::ItemImpl>(value.clone())?
        })
    }
}

impl ModuleIR {
    pub fn self_code(&self) -> &ItemImpl {
        &self.code
    }

    pub fn module_ident(&self) -> Result<Ident, syn::Error> {
        syn_utils::ident_from_impl(&self.code)
    }

    pub fn host_ref_ident(&self) -> Result<Ident, syn::Error> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}HostRef", module_ident),
            module_ident.span()
        ))
    }
    pub fn contract_ref_ident(&self) -> Result<Ident, syn::Error> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}ContractRef", module_ident),
            module_ident.span()
        ))
    }

    pub fn deployer_ident(&self) -> Result<Ident, syn::Error> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}Deployer", module_ident),
            module_ident.span()
        ))
    }

    pub fn functions(&self) -> Vec<FnIR> {
        self.code
            .items
            .iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(func) => Some(FnIR::new(func.clone())),
                _ => None
            })
            .collect::<Vec<_>>()
    }
}

pub struct FnIR {
    code: syn::ImplItemFn
}

impl FnIR {
    pub fn new(code: syn::ImplItemFn) -> Self {
        FnIR { code }
    }

    pub fn name(&self) -> Ident {
        self.code.sig.ident.clone()
    }

    pub fn try_name(&self) -> Ident {
        format_ident!("try_{}", self.name())
    }

    pub fn name_str(&self) -> String {
        self.name().to_string()
    }

    pub fn arg_names(&self) -> Vec<Ident> {
        syn_utils::function_arg_names(&self.code)
    }

    pub fn args_len(&self) -> usize {
        syn_utils::function_args(&self.code).len()
    }

    pub fn return_type(&self) -> syn::ReturnType {
        syn_utils::function_return_type(&self.code)
    }

    pub fn try_return_type(&self) -> syn::ReturnType {
        match self.return_type() {
            syn::ReturnType::Default => parse_quote!(-> Result<(), OdraError>),
            syn::ReturnType::Type(_, box ty) => parse_quote!(-> Result<#ty, OdraError>)
        }
    }

    pub fn typed_args(&self) -> Vec<syn::PatType> {
        syn_utils::function_args(&self.code)
    }

    pub fn is_mut(&self) -> bool {
        let receiver = syn_utils::receiver_arg(&self.code);
        receiver.map(|r| r.mutability.is_some()).unwrap_or_default()
    }
}

/// Intended to be used in [quote::ToTokens]. Emits error and ends item tokenization.
macro_rules! checked_unwrap {
    ($value:expr) => {
        match $value {
            Ok(result) => result,
            Err(e) => {
                proc_macro_error::emit_error!(e.span(), e.to_string());
                return;
            }
        }
    };
}
pub(crate) use checked_unwrap;
