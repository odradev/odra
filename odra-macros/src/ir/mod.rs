use crate::utils;
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use syn::{parse_quote, ItemImpl};

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
        utils::syn::ident_from_impl(&self.code)
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

    #[allow(dead_code)]
    pub fn deployer_ident(&self) -> Result<Ident, syn::Error> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}Deployer", module_ident),
            module_ident.span()
        ))
    }

    pub fn test_parts_mod_ident(&self) -> Result<syn::Ident, syn::Error> {
        self.module_ident()
            .map(odra_utils::camel_to_snake)
            .map(|ident| format_ident!("__{}_test_parts", ident))
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
        utils::syn::function_arg_names(&self.code)
    }

    #[allow(dead_code)]
    pub fn args_len(&self) -> usize {
        utils::syn::function_args(&self.code).len()
    }

    pub fn return_type(&self) -> syn::ReturnType {
        utils::syn::function_return_type(&self.code)
    }

    pub fn try_return_type(&self) -> syn::ReturnType {
        match self.return_type() {
            syn::ReturnType::Default => parse_quote!(-> Result<(), OdraError>),
            syn::ReturnType::Type(_, box ty) => parse_quote!(-> Result<#ty, OdraError>)
        }
    }

    pub fn typed_args(&self) -> Vec<syn::PatType> {
        utils::syn::function_args(&self.code)
    }

    pub fn is_mut(&self) -> bool {
        let receiver = utils::syn::receiver_arg(&self.code);
        receiver.map(|r| r.mutability.is_some()).unwrap_or_default()
    }
}
