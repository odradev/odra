use crate::utils;
use proc_macro2::Ident;
use quote::format_ident;
use syn::{parse_quote, spanned::Spanned};

const CONSTRUCTOR_NAME: &str = "init";

macro_rules! try_parse {
    ($from:path => $to:ident) => {
        pub struct $to {
            code: $from
        }

        impl TryFrom<&proc_macro2::TokenStream> for $to {
            type Error = syn::Error;

            fn try_from(stream: &proc_macro2::TokenStream) -> Result<Self, Self::Error> {
                Ok(Self {
                    code: syn::parse2::<$from>(stream.clone())?
                })
            }
        }
    };
}

try_parse!(syn::ItemStruct => StructIR);

impl StructIR {
    pub fn self_code(&self) -> &syn::ItemStruct {
        &self.code
    }

    pub fn field_names(&self) -> Result<Vec<syn::Ident>, syn::Error> {
        utils::syn::struct_fields_ident(&self.code)
    }

    pub fn module_ident(&self) -> syn::Ident {
        utils::syn::ident_from_struct(&self.code)
    }

    pub fn module_mod_ident(&self) -> syn::Ident {
        format_ident!(
            "__{}_module",
            utils::string::camel_to_snake(self.module_ident())
        )
    }

    pub fn typed_fields(&self) -> Result<Vec<EnumeratedTypedField>, syn::Error> {
        let fields = utils::syn::struct_fields(&self.code)?;
        let fields = fields
            .iter()
            .filter(|(i, _)| i != &utils::ident::env())
            .collect::<Vec<_>>();

        for (_, ty) in &fields {
            Self::validate_ty(ty)?;
        }

        fields
            .iter()
            .enumerate()
            .map(|(idx, (ident, ty))| {
                Ok(EnumeratedTypedField {
                    idx: idx as u8,
                    ident: ident.clone(),
                    ty: utils::syn::clear_generics(ty)?
                })
            })
            .collect()
    }

    fn validate_ty(ty: &syn::Type) -> Result<(), syn::Error> {
        let non_generic_ty = utils::syn::clear_generics(ty)?;

        // both odra::Variable and Variable (Mapping, ModuleWrapper) are valid.
        let valid_types = vec![
            utils::ty::module_wrapper(),
            utils::ty::variable(),
            utils::ty::mapping(),
        ]
        .iter()
        .map(|ty| utils::syn::last_segment_ident(ty).map(|i| vec![ty.clone(), parse_quote!(#i)]))
        .collect::<Result<Vec<_>, _>>()?;
        let valid_types = valid_types.into_iter().flatten().collect::<Vec<_>>();

        if valid_types
            .iter()
            .any(|t| utils::string::eq(t, &non_generic_ty))
        {
            return Ok(());
        }

        Err(syn::Error::new(ty.span(), "Invalid module type"))
    }
}

pub struct EnumeratedTypedField {
    pub idx: u8,
    pub ident: syn::Ident,
    pub ty: syn::Type
}

try_parse!(syn::ItemImpl => ModuleIR);

impl ModuleIR {
    pub fn self_code(&self) -> &syn::ItemImpl {
        &self.code
    }

    pub fn module_ident(&self) -> Result<Ident, syn::Error> {
        utils::syn::ident_from_impl(&self.code)
    }

    pub fn module_str(&self) -> Result<String, syn::Error> {
        self.module_ident().map(|i| i.to_string())
    }

    pub fn snake_cased_module_ident(&self) -> Result<Ident, syn::Error> {
        let ident = self.module_ident()?;
        Ok(Ident::new(
            utils::string::camel_to_snake(&ident).as_str(),
            ident.span()
        ))
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

    pub fn test_parts_mod_ident(&self) -> Result<syn::Ident, syn::Error> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_test_parts", module_ident),
            module_ident.span()
        ))
    }

    pub fn wasm_parts_mod_ident(&self) -> Result<syn::Ident, syn::Error> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_wasm_parts", module_ident),
            module_ident.span()
        ))
    }

    pub fn exec_parts_mod_ident(&self) -> Result<syn::Ident, syn::Error> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_exec_parts", module_ident),
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

    pub fn host_functions(&self) -> Vec<FnIR> {
        self.functions()
            .into_iter()
            .filter(|f| f.name_str() != CONSTRUCTOR_NAME)
            .collect()
    }

    pub fn constructor(&self) -> Option<FnIR> {
        self.functions()
            .into_iter()
            .find(|f| f.name_str() == CONSTRUCTOR_NAME)
    }

    pub fn constructor_args(&self) -> syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]> {
        self.constructor()
            .map(|f| {
                f.named_args()
                    .into_iter()
                    .map(|a| a.self_code().clone())
                    .collect()
            })
            .unwrap_or_default()
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

    pub fn execute_name(&self) -> Ident {
        format_ident!("execute_{}", self.name())
    }

    pub fn name_str(&self) -> String {
        self.name().to_string()
    }

    pub fn arg_names(&self) -> Vec<Ident> {
        utils::syn::function_arg_names(&self.code)
    }

    pub fn named_args(&self) -> Vec<FnArgIR> {
        utils::syn::function_named_args(&self.code)
            .into_iter()
            .map(|arg| FnArgIR::new(arg.to_owned()))
            .collect()
    }

    pub fn return_type(&self) -> syn::ReturnType {
        utils::syn::function_return_type(&self.code)
    }

    pub fn try_return_type(&self) -> syn::ReturnType {
        let ty_odra_err = utils::ty::odra_error();
        match self.return_type() {
            syn::ReturnType::Default => parse_quote!(-> Result<(), #ty_odra_err>),
            syn::ReturnType::Type(_, box ty) => parse_quote!(-> Result<#ty, #ty_odra_err>)
        }
    }

    pub fn typed_args(&self) -> Vec<syn::PatType> {
        utils::syn::function_typed_args(&self.code)
    }

    pub fn is_mut(&self) -> bool {
        let receiver = utils::syn::receiver_arg(&self.code);
        receiver.map(|r| r.mutability.is_some()).unwrap_or_default()
    }

    pub fn is_constructor(&self) -> bool {
        self.name_str() == CONSTRUCTOR_NAME
    }
}

pub struct FnArgIR {
    code: syn::FnArg
}

impl FnArgIR {
    pub fn new(code: syn::FnArg) -> Self {
        FnArgIR { code }
    }

    pub fn self_code(&self) -> &syn::FnArg {
        &self.code
    }

    pub fn name(&self) -> Result<Ident, syn::Error> {
        match &self.code {
            syn::FnArg::Typed(syn::PatType {
                pat: box syn::Pat::Ident(pat),
                ..
            }) => Ok(pat.ident.clone()),
            _ => Err(syn::Error::new_spanned(&self.code, "Unnamed arg"))
        }
    }

    pub fn name_and_ty(&self) -> Result<(String, syn::Type), syn::Error> {
        match &self.code {
            syn::FnArg::Typed(syn::PatType {
                box ty,
                pat: box syn::Pat::Ident(pat),
                ..
            }) => Ok((pat.ident.to_string(), ty.clone())),
            _ => Err(syn::Error::new_spanned(&self.code, "Unnamed arg"))
        }
    }
}
