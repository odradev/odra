use proc_macro2::Ident;
use quote::format_ident;
use syn::parse_quote;

use super::attr::{self, OdraAttribute};
use super::errors;
use super::{
    FnType, BATCH_FACTORY_UPGRADE_NAME, CONSTRUCTOR_NAME, FACTORY_NAME, FACTORY_UPGRADE_NAME,
    UPGRADER_NAME
};
use crate::utils;

/// A function of a module impl block (`Impl`) or of a trait (`Def`): an entry point, or an
/// `#[odra(offchain)]` function run on the host.
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

    pub fn fn_type(&self) -> FnType {
        let name = self.name_str();
        match name.as_str() {
            CONSTRUCTOR_NAME => FnType::Constructor,
            UPGRADER_NAME => FnType::Upgrader,
            FACTORY_NAME => FnType::Factory,
            FACTORY_UPGRADE_NAME => FnType::FactoryUpgrader,
            BATCH_FACTORY_UPGRADE_NAME => FnType::FactoryBatchUpgrader,
            _ => FnType::Regular
        }
    }
}

const PROTECTED_FUNCTIONS: [&str; 6] = [
    "new",
    "env",
    "address",
    "new_contract",
    "upgrade_child_contract",
    "batch_upgrade_child_contract"
];
const PROTECTED_ARGS: [&str; 2] = ["gas", "attached_value"];

fn validate_fn_name(name: &Ident) -> syn::Result<()> {
    if PROTECTED_FUNCTIONS.contains(&name.to_string().as_str()) {
        return Err(errors::reserved_fn_name(name));
    }
    Ok(())
}

/// An argument is `name: Type` (the receiver aside), and the name is not one every call carries.
fn validate_fn_arg(arg: &syn::FnArg) -> syn::Result<()> {
    let name = match arg {
        syn::FnArg::Receiver(_) => return Ok(()),
        syn::FnArg::Typed(syn::PatType { pat, .. }) => match pat.as_ref() {
            syn::Pat::Ident(pat) => &pat.ident,
            _ => return Err(errors::arg_without_name(arg))
        }
    };
    if PROTECTED_ARGS.contains(&name.to_string().as_str()) {
        return Err(errors::reserved_arg_name(name));
    }
    Ok(())
}

/// Functions whose wasm entry point is built by a dedicated codegen path
/// (see `AddEntryPointStmtItem`) that does not read the `#[odra(..)]` flags.
/// An attribute here would be dropped without a trace, so it is rejected instead.
const NO_ODRA_ATTRIBUTE_FUNCTIONS: [&str; 2] = [CONSTRUCTOR_NAME, UPGRADER_NAME];

fn validate_fn_attrs(fn_name: &str, attrs: &[syn::Attribute]) -> syn::Result<()> {
    if !NO_ODRA_ATTRIBUTE_FUNCTIONS.contains(&fn_name) {
        return Ok(());
    }

    errors::check_all(
        attrs
            .iter()
            .filter(|attr| attr.path().is_ident("odra"))
            .map(|attr| {
                let odra_attr = OdraAttribute::try_from(attr.clone())?;
                match odra_attr.arg_names().first() {
                    Some(arg) => Err(errors::odra_attr_on_special_fn(attr, arg, fn_name)),
                    None => Ok(())
                }
            })
    )
}

impl TryFrom<syn::TraitItemFn> for FnIR {
    type Error = syn::Error;

    fn try_from(code: syn::TraitItemFn) -> Result<Self, Self::Error> {
        let fn_name = utils::syn::function_name(&code.sig);
        validate_fn_ir(&fn_name, &code.sig, &code.attrs)?;
        Ok(Self::Def(FnTraitIR::new(code)))
    }
}

impl TryFrom<syn::ImplItemFn> for FnIR {
    type Error = syn::Error;

    fn try_from(code: syn::ImplItemFn) -> Result<Self, Self::Error> {
        let fn_name = utils::syn::function_name(&code.sig);
        validate_fn_ir(&fn_name, &code.sig, &code.attrs)?;
        Ok(Self::Impl(FnImplIR::new(code)))
    }
}

fn validate_fn_ir(
    fn_name: &str,
    sig: &syn::Signature,
    attrs: &[syn::Attribute]
) -> syn::Result<()> {
    errors::check_all([
        validate_fn_name(&sig.ident),
        errors::check_all(sig.inputs.iter().map(validate_fn_arg)),
        attr::partition_attributes(attrs.to_vec()).map(|_| ()),
        validate_fn_attrs(fn_name, attrs)
    ])
}

impl FnIR {
    pub fn name(&self) -> Ident {
        self.sig().ident.clone()
    }

    pub fn try_name(&self) -> Ident {
        format_ident!("try_{}", self.name())
    }

    pub fn try_no_ret_name(&self) -> Ident {
        format_ident!("try_{}_no_ret", self.name())
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

    pub fn is_offchain(&self) -> bool {
        let (odra_attrs, _) = attr::partition_attributes(self.attrs()).unwrap_or_default();
        odra_attrs.iter().any(OdraAttribute::is_offchain)
    }

    pub fn arg_names(&self) -> Vec<Ident> {
        self.named_args().iter().map(FnArgIR::name).collect()
    }

    pub fn has_args(&self) -> bool {
        !self.arg_names().is_empty()
    }

    pub fn named_args(&self) -> Vec<FnArgIR> {
        self.sig()
            .inputs
            .iter()
            .filter_map(FnArgIR::named)
            .collect()
    }

    pub fn return_type(&self) -> syn::ReturnType {
        utils::syn::function_return_type(self.sig())
    }

    pub fn try_return_type(&self) -> syn::ReturnType {
        let ty = match self.return_type() {
            syn::ReturnType::Default => parse_quote!(()),
            syn::ReturnType::Type(_, ty) => parse_quote!(#ty)
        };
        let odra_result = utils::ty::odra_result(ty);
        utils::misc::ret_ty(&odra_result)
    }

    pub fn typed_args(&self) -> Vec<syn::PatType> {
        utils::syn::function_typed_args(self.sig())
    }

    pub fn generics(&self) -> &syn::Generics {
        &self.sig().generics
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

    pub fn is_restricted(&self) -> bool {
        self.is_constructor() || self.is_upgrader()
    }

    pub fn is_constructor(&self) -> bool {
        self.name_str() == CONSTRUCTOR_NAME
    }

    pub fn is_upgrader(&self) -> bool {
        self.name_str() == UPGRADER_NAME
    }

    pub fn is_factory(&self) -> bool {
        self.name_str() == "factory"
    }

    /// The submodule field a `delegate!`-generated function with no docs of its own forwards
    /// to; its schema description is inherited from that submodule's entry point.
    pub fn inherits_description_from(&self) -> Option<&syn::ExprField> {
        match self {
            FnIR::Impl(ir) => ir
                .delegation
                .as_ref()
                .filter(|delegation| !delegation.has_own_docs)
                .map(|delegation| &delegation.target),
            FnIR::Def(_) => None
        }
    }

    fn attrs(&self) -> Vec<syn::Attribute> {
        match self {
            FnIR::Impl(ir) => ir.attrs(),
            FnIR::Def(ir) => ir.attrs()
        }
        .to_vec()
    }

    pub(super) fn sig(&self) -> &syn::Signature {
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

    pub fn sig(&self) -> &syn::Signature {
        &self.code.sig
    }

    fn attrs(&self) -> &[syn::Attribute] {
        &self.code.attrs
    }
}

/// Where a function generated by a `delegate!` block forwards its call.
pub struct Delegation {
    /// The submodule field, e.g. `self.metadata`.
    pub target: syn::ExprField,
    /// `false` if the docs were generated, so the schema description is inherited
    /// from the submodule's entry point of the same name.
    pub has_own_docs: bool
}

pub struct FnImplIR {
    code: syn::ImplItemFn,
    delegation: Option<Delegation>
}

impl FnImplIR {
    pub fn new(code: syn::ImplItemFn) -> Self {
        Self {
            code,
            delegation: None
        }
    }

    pub(super) fn delegated_to(self, target: syn::ExprField, has_own_docs: bool) -> Self {
        Self {
            delegation: Some(Delegation {
                target,
                has_own_docs
            }),
            ..self
        }
    }

    pub fn sig(&self) -> &syn::Signature {
        &self.code.sig
    }

    fn attrs(&self) -> &[syn::Attribute] {
        &self.code.attrs
    }

    pub fn raw(&self) -> syn::ImplItemFn {
        self.code.clone()
    }
}

/// A `name: Type` argument of an entry point. Only such arguments reach here: a pattern in
/// place of the name is rejected when the function is read.
pub struct FnArgIR {
    ident: Ident,
    code: syn::PatType
}

impl FnArgIR {
    /// The argument, when it is `name: Type`.
    fn named(arg: &syn::FnArg) -> Option<Self> {
        let syn::FnArg::Typed(code) = arg else {
            return None;
        };
        let syn::Pat::Ident(pat) = code.pat.as_ref() else {
            return None;
        };
        Some(Self {
            ident: pat.ident.clone(),
            code: code.clone()
        })
    }

    pub fn raw(&self) -> syn::FnArg {
        syn::FnArg::Typed(self.code.clone())
    }

    pub fn name(&self) -> Ident {
        self.ident.clone()
    }

    pub fn name_str(&self) -> String {
        self.ident.to_string()
    }

    pub fn ty(&self) -> &syn::Type {
        &self.code.ty
    }

    pub fn name_and_ty(&self) -> (String, syn::Type) {
        (self.name_str(), self.ty().clone())
    }

    pub fn is_ref(&self) -> bool {
        utils::syn::is_ref(self.ty())
    }
}
