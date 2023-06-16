use crate::generator::{self, GenerateCode};

/// Types which can generate code.
pub trait OdraPoet: Sized {
    /// The underlying generator generating the code.
    type Poet: From<Self> + GenerateCode;
}

impl<'a> OdraPoet for &'a odra_ir::module::ModuleImpl {
    type Poet = generator::ModuleImpl<'a>;
}

impl<'a> OdraPoet for &'a odra_ir::module::ModuleStruct {
    type Poet = generator::ModuleStruct<'a>;
}

impl<'a> OdraPoet for &'a odra_ir::EventItem {
    type Poet = generator::event_item::EventItem<'a>;
}

impl<'a> OdraPoet for &'a odra_ir::OdraTypeItem {
    type Poet = generator::odra_type_item::OdraTypeItem<'a>;
}

impl<'a> OdraPoet for &'a odra_ir::InstanceItem {
    type Poet = generator::instance_item::InstanceItem<'a>;
}

impl<'a> OdraPoet for &'a odra_ir::ExternalContractItem {
    type Poet = generator::external_contract_item::ExternalContractItem<'a>;
}

impl<'a> OdraPoet for &'a odra_ir::ErrorEnumItem {
    type Poet = generator::errors::ErrorEnumItem<'a>;
}

impl<'a> OdraPoet for &'a syn::ItemEnum {
    type Poet = generator::errors::OdraErrorItem<'a>;
}

impl<'a> OdraPoet for &'a odra_ir::MapExpr {
    type Poet = generator::mapping::OdraMapping<'a>;
}

pub trait OdraPoetUsingImpl: AsRef<odra_ir::module::ModuleImpl> {
    fn generate_code_using<'a, G>(&'a self) -> proc_macro2::TokenStream
    where
        G: GenerateCode + From<&'a odra_ir::module::ModuleImpl>;
}

impl<T> OdraPoetUsingImpl for T
where
    T: AsRef<odra_ir::module::ModuleImpl>
{
    fn generate_code_using<'a, G>(&'a self) -> proc_macro2::TokenStream
    where
        G: GenerateCode + From<&'a odra_ir::module::ModuleImpl>
    {
        <G as GenerateCode>::generate_code(&G::from(
            <Self as AsRef<odra_ir::module::ModuleImpl>>::as_ref(self)
        ))
    }
}

pub trait OdraPoetUsingStruct: AsRef<odra_ir::module::ModuleStruct> {
    fn generate_code_using<'a, G>(&'a self) -> proc_macro2::TokenStream
    where
        G: GenerateCode + From<&'a odra_ir::module::ModuleStruct>;
}

impl<T> OdraPoetUsingStruct for T
where
    T: AsRef<odra_ir::module::ModuleStruct>
{
    fn generate_code_using<'a, G>(&'a self) -> proc_macro2::TokenStream
    where
        G: GenerateCode + From<&'a odra_ir::module::ModuleStruct>
    {
        <G as GenerateCode>::generate_code(&G::from(
            <Self as AsRef<odra_ir::module::ModuleStruct>>::as_ref(self)
        ))
    }
}
