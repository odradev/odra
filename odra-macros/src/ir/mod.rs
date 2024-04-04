use std::collections::HashSet;

use crate::ir::delegate::Delegate;
use crate::utils;
use config::ConfigItem;
use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::{parse_quote, spanned::Spanned, ImplItem};

use self::attr::OdraAttribute;

mod attr;
mod config;
pub mod delegate;

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

try_parse!(syn::ItemStruct => ModuleStructIR);

impl ModuleStructIR {
    pub fn self_code(&self) -> &syn::ItemStruct {
        &self.code
    }

    pub fn field_names(&self) -> syn::Result<Vec<syn::Ident>> {
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

    pub fn contract_schema_mod_ident(&self) -> Ident {
        let ident = self.module_ident();
        Ident::new(
            utils::string::camel_to_snake(format!("__{}_contract_schema", ident)).as_str(),
            ident.span()
        )
    }

    pub fn typed_fields(&self) -> syn::Result<Vec<EnumeratedTypedField>> {
        let fields = utils::syn::struct_typed_fields(&self.code)?;
        let fields = fields
            .iter()
            .filter(|(i, _)| i != &utils::ident::env())
            .collect::<Vec<_>>();

        fields
            .iter()
            .enumerate()
            .map(|(idx, (ident, ty))| {
                Ok(EnumeratedTypedField {
                    idx: idx as u8,
                    ident: ident.clone(),
                    ty: ty.clone()
                })
            })
            .collect()
    }

    pub fn events(&self) -> Vec<syn::Type> {
        if let ConfigItem::Module(cfg) = &self.config {
            cfg.events.iter().cloned().collect()
        } else {
            vec![]
        }
    }

    pub fn errors(&self) -> Option<syn::Type> {
        if let ConfigItem::Module(cfg) = &self.config {
            (*cfg.errors).clone()
        } else {
            None
        }
    }

    pub fn contract_name(&self) -> String {
        if let ConfigItem::Module(cfg) = &self.config {
            (*cfg.name).clone()
        } else {
            String::from("")
        }
    }

    pub fn contract_version(&self) -> String {
        if let ConfigItem::Module(cfg) = &self.config {
            (*cfg.version).clone()
        } else {
            String::from("")
        }
    }

    pub fn unique_fields_ty(&self) -> syn::Result<Vec<syn::Type>> {
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
        let fields = utils::syn::struct_typed_fields(&self.code)?;
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
}

pub struct EnumeratedTypedField {
    pub idx: u8,
    pub ident: syn::Ident,
    pub ty: syn::Type
}

pub enum ModuleImplIR {
    Impl(ModuleIR),
    Trait(ModuleTraitIR)
}

impl TryFrom<(&proc_macro2::TokenStream, &proc_macro2::TokenStream)> for ModuleImplIR {
    type Error = syn::Error;

    fn try_from(
        stream: (&proc_macro2::TokenStream, &proc_macro2::TokenStream)
    ) -> Result<Self, Self::Error> {
        let config = syn::parse2::<ConfigItem>(stream.0.clone())?;
        if let Ok(code) = syn::parse2::<syn::ItemImpl>(stream.1.clone()) {
            return Ok(Self::Impl(ModuleIR { code, config }));
        }

        if let Ok(code) = syn::parse2::<syn::ItemTrait>(stream.1.clone()) {
            return Ok(Self::Trait(ModuleTraitIR { code, config }));
        }

        Err(syn::Error::new_spanned(
            stream.1,
            "Impl or Trait block expected"
        ))
    }
}

impl ModuleImplIR {
    pub fn self_code(&self) -> proc_macro2::TokenStream {
        match self {
            ModuleImplIR::Impl(ir) => ir.self_code().into_token_stream(),
            ModuleImplIR::Trait(ir) => ir.self_code().into_token_stream()
        }
    }

    pub fn module_ident(&self) -> syn::Result<Ident> {
        match self {
            ModuleImplIR::Impl(ir) => utils::syn::ident_from_impl(&ir.code),
            ModuleImplIR::Trait(ir) => Ok(ir.module_ident())
        }
    }

    pub fn is_trait_impl(&self) -> bool {
        match self {
            ModuleImplIR::Impl(ir) => ir.code.trait_.is_some(),
            ModuleImplIR::Trait(_) => false
        }
    }

    pub fn impl_trait_ident(&self) -> Option<Ident> {
        match self {
            ModuleImplIR::Impl(ir) => ir
                .code
                .trait_
                .clone()
                .map(|(_, path, _)| path.get_ident().unwrap().clone()),
            ModuleImplIR::Trait(_) => None
        }
    }

    pub fn module_str(&self) -> syn::Result<String> {
        self.module_ident().map(|i| i.to_string())
    }

    pub fn snake_cased_module_ident(&self) -> syn::Result<Ident> {
        let ident = self.module_ident()?;
        Ok(Ident::new(
            utils::string::camel_to_snake(&ident).as_str(),
            ident.span()
        ))
    }

    pub fn host_ref_ident(&self) -> syn::Result<Ident> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}HostRef", module_ident),
            module_ident.span()
        ))
    }
    pub fn contract_ref_ident(&self) -> syn::Result<Ident> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}ContractRef", module_ident),
            module_ident.span()
        ))
    }

    pub fn init_args_ident(&self) -> syn::Result<syn::Ident> {
        let module_ident = self.module_ident()?;
        Ok(Ident::new(
            &format!("{}InitArgs", module_ident),
            module_ident.span()
        ))
    }

    pub fn schema_mod_ident(&self) -> syn::Result<Ident> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_schema", module_ident),
            module_ident.span()
        ))
    }

    pub fn test_parts_mod_ident(&self) -> syn::Result<syn::Ident> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_test_parts", module_ident),
            module_ident.span()
        ))
    }

    pub fn wasm_parts_mod_ident(&self) -> syn::Result<syn::Ident> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_wasm_parts", module_ident),
            module_ident.span()
        ))
    }

    pub fn exec_parts_mod_ident(&self) -> syn::Result<syn::Ident> {
        let module_ident = self.snake_cased_module_ident()?;
        Ok(Ident::new(
            &format!("__{}_exec_parts", module_ident),
            module_ident.span()
        ))
    }

    pub fn host_functions(&self) -> syn::Result<Vec<FnIR>> {
        Ok(self.functions()?.into_iter().collect())
    }

    pub fn constructor(&self) -> Option<FnIR> {
        self.functions()
            .unwrap_or_default()
            .into_iter()
            .find(|f| f.name_str() == CONSTRUCTOR_NAME)
    }

    pub fn functions(&self) -> syn::Result<Vec<FnIR>> {
        match self {
            ModuleImplIR::Impl(ir) => ir.functions(),
            ModuleImplIR::Trait(ir) => ir.functions()
        }
    }
}

try_parse!(syn::ItemImpl => ModuleIR);

impl ModuleIR {
    fn self_code(&self) -> syn::ItemImpl {
        let mut code = self.code.clone();
        // include delegated functions
        code.items.extend(
            self.delegated_functions()
                .unwrap_or_default()
                .into_iter()
                .map(syn::ImplItem::Fn)
        );
        // remove odra attributes
        code.items.iter_mut().for_each(|item| {
            if let syn::ImplItem::Fn(func) = item {
                func.attrs = attr::other_attributes(func.attrs.clone());
            }
        });
        // remove inner odra macros
        code.items
            .retain(|item| !matches!(item, syn::ImplItem::Macro(_)));
        code
    }

    fn functions(&self) -> syn::Result<Vec<FnIR>> {
        self.code
            .items
            .clone()
            .into_iter()
            .filter_map(|item| match item {
                syn::ImplItem::Fn(func) => Some(func),
                _ => None
            })
            .chain(self.delegated_functions().unwrap_or_default())
            .map(FnIR::try_from)
            .filter(|r| self.is_trait_impl() || r.as_ref().map(FnIR::is_pub).unwrap_or(true))
            .collect::<Result<Vec<_>, _>>()
    }

    fn delegated_functions(&self) -> syn::Result<Vec<syn::ImplItemFn>> {
        let macro_item = self.code.items.iter().find_map(|item| match item {
            ImplItem::Macro(m) => Some(m),
            _ => None
        });
        if let Some(item) = macro_item {
            return Ok(syn::parse2::<Delegate>(item.mac.tokens.clone())?.functions);
        }

        Ok(vec![])
    }

    fn is_trait_impl(&self) -> bool {
        self.code.trait_.is_some()
    }
}

try_parse!(syn::ItemTrait => ModuleTraitIR);

impl ModuleTraitIR {
    fn self_code(&self) -> syn::ItemTrait {
        let mut code = self.code.clone();
        code.items.iter_mut().for_each(|item| {
            if let syn::TraitItem::Fn(func) = item {
                func.attrs = attr::other_attributes(func.attrs.clone());
            }
        });
        code
    }

    fn module_ident(&self) -> syn::Ident {
        self.code.ident.clone()
    }

    fn functions(&self) -> syn::Result<Vec<FnIR>> {
        self.code
            .items
            .iter()
            .filter_map(|item| match item {
                syn::TraitItem::Fn(func) => Some(FnIR::try_from(func.clone())),
                _ => None
            })
            .collect()
    }
}

pub enum FnIR {
    Impl(FnImplIR),
    Def(FnTraitIR)
}

impl FnIR {
    pub fn attributes(&self) -> &[syn::Attribute] {
        match self {
            FnIR::Impl(ir) => ir.attrs(),
            FnIR::Def(ir) => ir.attrs()
        }
    }

    pub fn docs(&self) -> Vec<String> {
        utils::syn::string_docs(self.attributes())
    }
}

const PROTECTED_FUNCTIONS: [&str; 3] = ["new", "env", "address"];

fn validate_fn_name<T: ToTokens>(name: &str, ctx: T) -> syn::Result<()> {
    if PROTECTED_FUNCTIONS.contains(&name) {
        return Err(syn::Error::new_spanned(
            ctx,
            format!("Entrypoint name `{}` is reserved", name)
        ));
    }
    Ok(())
}

impl TryFrom<syn::TraitItemFn> for FnIR {
    type Error = syn::Error;

    fn try_from(code: syn::TraitItemFn) -> Result<Self, Self::Error> {
        let fn_name = utils::syn::function_name(&code.sig);
        validate_fn_name(&fn_name, &code)?;
        Ok(Self::Def(FnTraitIR::new(code)))
    }
}

impl TryFrom<syn::ImplItemFn> for FnIR {
    type Error = syn::Error;

    fn try_from(code: syn::ImplItemFn) -> Result<Self, Self::Error> {
        let fn_name = utils::syn::function_name(&code.sig);
        validate_fn_name(&fn_name, &code)?;
        Ok(Self::Impl(FnImplIR::new(code)))
    }
}

impl FnIR {
    pub fn name(&self) -> Ident {
        self.sig().ident.clone()
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
        let (odra_attrs, _) = attr::partition_attributes(self.attrs()).unwrap_or_default();
        odra_attrs.iter().any(OdraAttribute::is_payable)
    }

    pub fn is_non_reentrant(&self) -> bool {
        let (odra_attrs, _) = attr::partition_attributes(self.attrs()).unwrap_or_default();
        odra_attrs.iter().any(OdraAttribute::is_non_reentrant)
    }

    pub fn arg_names(&self) -> Vec<Ident> {
        utils::syn::function_arg_names(self.sig())
    }

    pub fn has_args(&self) -> bool {
        !self.arg_names().is_empty()
    }

    pub fn named_args(&self) -> Vec<FnArgIR> {
        utils::syn::function_named_args(self.sig())
            .into_iter()
            .map(|arg| FnArgIR::new(arg.to_owned()))
            .collect()
    }

    pub fn return_type(&self) -> syn::ReturnType {
        utils::syn::function_return_type(self.sig())
    }

    pub fn try_return_type(&self) -> syn::ReturnType {
        let ty = match self.return_type() {
            syn::ReturnType::Default => parse_quote!(()),
            syn::ReturnType::Type(_, box ty) => parse_quote!(#ty)
        };
        let odra_result = utils::ty::odra_result(ty);
        utils::misc::ret_ty(&odra_result)
    }

    pub fn typed_args(&self) -> Vec<syn::PatType> {
        utils::syn::function_typed_args(self.sig())
    }

    pub fn raw_typed_args(&self) -> Vec<syn::PatType> {
        self.typed_args()
            .into_iter()
            .map(|pat_ty| syn::PatType {
                ty: Box::new(utils::ty::unreferenced_ty(&pat_ty.ty)),
                ..pat_ty
            })
            .collect()
    }

    pub fn is_mut(&self) -> bool {
        let receiver = utils::syn::receiver_arg(self.sig());
        receiver.map(|r| r.mutability.is_some()).unwrap_or_default()
    }

    pub fn is_constructor(&self) -> bool {
        self.name_str() == CONSTRUCTOR_NAME
    }

    pub fn is_pub(&self) -> bool {
        match self {
            FnIR::Impl(ir) => ir.is_pub(),
            FnIR::Def(_) => true
        }
    }

    fn attrs(&self) -> Vec<syn::Attribute> {
        match self {
            FnIR::Impl(ir) => ir.attrs(),
            FnIR::Def(ir) => ir.attrs()
        }
        .to_vec()
    }

    fn sig(&self) -> &syn::Signature {
        match self {
            FnIR::Impl(ir) => ir.sig(),
            FnIR::Def(ir) => ir.sig()
        }
    }
}

pub struct FnTraitIR {
    code: syn::TraitItemFn
}

impl FnTraitIR {
    pub fn new(code: syn::TraitItemFn) -> Self {
        Self { code }
    }

    fn sig(&self) -> &syn::Signature {
        &self.code.sig
    }

    fn attrs(&self) -> &[syn::Attribute] {
        &self.code.attrs
    }
}

pub struct FnImplIR {
    code: syn::ImplItemFn
}

impl FnImplIR {
    pub fn new(code: syn::ImplItemFn) -> Self {
        Self { code }
    }

    fn sig(&self) -> &syn::Signature {
        &self.code.sig
    }

    fn attrs(&self) -> &[syn::Attribute] {
        &self.code.attrs
    }

    fn is_pub(&self) -> bool {
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

    pub fn name(&self) -> syn::Result<Ident> {
        match &self.code {
            syn::FnArg::Typed(syn::PatType {
                pat: box syn::Pat::Ident(pat),
                ..
            }) => Ok(pat.ident.clone()),
            _ => Err(syn::Error::new_spanned(&self.code, "Unnamed arg"))
        }
    }

    pub fn name_str(&self) -> syn::Result<String> {
        self.name().map(|i| i.to_string())
    }

    pub fn ty(&self) -> syn::Result<syn::Type> {
        match &self.code {
            syn::FnArg::Typed(syn::PatType { box ty, .. }) => Ok(ty.clone()),
            _ => Err(syn::Error::new_spanned(&self.code, "Unnamed arg"))
        }
    }
    pub fn name_and_ty(&self) -> syn::Result<(String, syn::Type)> {
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
    code: syn::Item
}

impl TryFrom<&proc_macro2::TokenStream> for TypeIR {
    type Error = syn::Error;

    fn try_from(stream: &proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        Ok(Self {
            code: syn::parse2::<syn::Item>(stream.clone())?
        })
    }
}

impl TypeIR {
    pub fn self_code(&self) -> &syn::Item {
        &self.code
    }

    pub fn kind(&self) -> syn::Result<TypeKind> {
        match &self.code {
            syn::Item::Enum(e) => {
                let is_unit = e.variants.iter().all(|v| v.fields.is_empty());
                let variants = e.variants.iter().cloned().collect();
                if is_unit {
                    Ok(TypeKind::UnitEnum { variants })
                } else {
                    Ok(TypeKind::Enum { variants })
                }
            }
            syn::Item::Struct(syn::ItemStruct { fields, .. }) => {
                let fields = fields
                    .iter()
                    .map(|f| {
                        f.ident
                            .clone()
                            .map(|i| (i, f.ty.clone()))
                            .ok_or(syn::Error::new(f.span(), "Unnamed field"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TypeKind::Struct { fields })
            }
            _ => Err(syn::Error::new_spanned(
                &self.code,
                "Invalid type. Only enums and structs are supported"
            ))
        }
    }
}

pub enum TypeKind {
    UnitEnum {
        variants: Vec<syn::Variant>
    },
    Enum {
        variants: Vec<syn::Variant>
    },
    Struct {
        fields: Vec<(syn::Ident, syn::Type)>
    }
}
