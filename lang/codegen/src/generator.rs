mod common;
pub mod errors;
pub mod event_item;
pub mod external_contract_item;
pub mod instance_item;
pub mod mapping;
pub mod module_impl;
pub mod module_item;

pub use {module_impl::ModuleImpl, module_item::ModuleStruct};

pub trait GenerateCode {
    fn generate_code(&self) -> proc_macro2::TokenStream;
}
