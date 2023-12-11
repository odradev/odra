use std::collections::HashSet;

use crate::utils;
use config::ConfigItem;
use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::{parse_quote, spanned::Spanned, Data};

use self::attr::OdraAttribute;

mod attr;
mod config;

const CONSTRUCTOR_NAME: &str = "init";

macro_rules! try_parse {
    ($from:path => $to:ident) => {
        pub struct $to {
            code: $from,
            #[allow(dead_code)]
            config: ConfigItem
        }

        impl TryFrom<(&proc_macro2::TokenStream, &proc_macro2::TokenStream)> for $to {
            type Error = syn::Error;

            fn try_from(
                stream: (&proc_macro2::TokenStream, &proc_macro2::TokenStream)
            ) -> Result<Self, Self::Error> {
                Ok(Self {
                    code: syn::parse2::<$from>(stream.1.clone())?,
                    config: syn::parse2::<ConfigItem>(stream.0.clone())?
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

    pub fn module_str(&self) -> String {
        self.module_ident().to_string()
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

    pub fn events(&self) -> Vec<syn::Type> {
        if let ConfigItem::Module(cfg) = &self.config {
            cfg.events.iter().map(|ev| ev.ty.clone()).collect()
        } else {
            vec![]
        }
    }

    pub fn unique_fields_ty(&self) -> Result<Vec<syn::Type>, syn::Error> {
        // A hack to sort types by their string representation. Otherwise, we would get an unstable
        // order of types in the generated code and tests would fail.
        #[derive(Eq, PartialEq)]
        struct OrdType(syn::Type);

        impl PartialOrd for OrdType {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for OrdType {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0
                    .to_token_stream()
                    .to_string()
                    .cmp(&other.0.to_token_stream().to_string())
            }
        }
        let fields = utils::syn::struct_fields(&self.code)?;
        let set = HashSet::<syn::Type>::from_iter(
            fields
                .iter()
                .filter(|(i, _)| i != &utils::ident::env())
                .map(|(_, ty)| ty.clone())
        );
        let mut fields = set.into_iter().map(OrdType).collect::<Vec<_>>();
        fields.sort();

        Ok(fields.into_iter().map(|i| i.0).collect())
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

// TODO: change the name
impl ModuleIR {
    pub fn self_code(&self) -> syn::ItemImpl {
        let mut code = self.code.clone();
        code.items.iter_mut().for_each(|item| {
            if let syn::ImplItem::Fn(func) = item {
                func.attrs = attr::other_attributes(func.attrs.clone());
            }
        });
        code
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

    pub fn schema_mod_ident(&self) -> Result<Ident, syn::Error> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_schema", module_ident),
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
            .filter(|f| self.is_trait_impl() || f.is_pub())
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

    fn is_trait_impl(&self) -> bool {
        self.code.trait_.is_some()
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

    pub fn is_payable(&self) -> bool {
        let (odra_attrs, _) =
            attr::partition_attributes(self.code.attrs.clone()).unwrap_or_default();
        odra_attrs.iter().any(OdraAttribute::is_payable)
    }

    pub fn is_non_reentrant(&self) -> bool {
        let (odra_attrs, _) =
            attr::partition_attributes(self.code.attrs.clone()).unwrap_or_default();
        odra_attrs.iter().any(OdraAttribute::is_non_reentrant)
    }

    pub fn arg_names(&self) -> Vec<Ident> {
        utils::syn::function_arg_names(&self.code)
    }

    pub fn has_args(&self) -> bool {
        !self.arg_names().is_empty()
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

    pub fn raw_typed_args(&self) -> Vec<syn::PatType> {
        self.typed_args()
            .into_iter()
            .map(|pat_ty| syn::PatType {
                ty: Box::new(utils::syn::unreferenced_ty(&pat_ty.ty)),
                ..pat_ty
            })
            .collect()
    }

    pub fn is_mut(&self) -> bool {
        let receiver = utils::syn::receiver_arg(&self.code);
        receiver.map(|r| r.mutability.is_some()).unwrap_or_default()
    }

    pub fn is_constructor(&self) -> bool {
        self.name_str() == CONSTRUCTOR_NAME
    }

    pub fn is_pub(&self) -> bool {
        matches!(self.code.vis, syn::Visibility::Public(_))
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

    pub fn name_str(&self) -> Result<String, syn::Error> {
        self.name().map(|i| i.to_string())
    }

    pub fn ty(&self) -> Result<syn::Type, syn::Error> {
        match &self.code {
            syn::FnArg::Typed(syn::PatType { box ty, .. }) => Ok(ty.clone()),
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

    pub fn is_ref(&self) -> bool {
        match &self.code {
            syn::FnArg::Typed(syn::PatType { box ty, .. }) => utils::syn::is_ref(ty),
            _ => false
        }
    }
}

pub struct TypeIR {
    code: syn::DeriveInput
}

impl TryFrom<&proc_macro2::TokenStream> for TypeIR {
    type Error = syn::Error;

    fn try_from(stream: &proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        Ok(Self {
            code: syn::parse2::<syn::DeriveInput>(stream.clone())?
        })
    }
}

impl TypeIR {
    pub fn self_code(&self) -> &syn::DeriveInput {
        &self.code
    }

    pub fn fields(&self) -> Result<Vec<syn::Ident>, syn::Error> {
        utils::syn::derive_item_variants(&self.code)
    }

    pub fn map_fields<F, R>(&self, func: F) -> Result<Vec<R>, syn::Error>
    where
        F: FnMut(&syn::Ident) -> R
    {
        Ok(self.fields()?.iter().map(func).collect::<Vec<_>>())
    }

    pub fn is_enum(&self) -> bool {
        matches!(self.code.data, Data::Enum(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self.code.data, Data::Struct(_))
    }
}
