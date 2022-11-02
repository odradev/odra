macro_rules! as_ref_for_contract_impl_generator {
    ($struct_ident:ident) => {
        impl ::core::convert::AsRef<odra_ir::module::ModuleImpl> for $struct_ident<'_> {
            fn as_ref(&self) -> &odra_ir::module::ModuleImpl {
                self.contract
            }
        }
    };
}

use generator::GenerateCode;
mod generator;
mod poet;
pub use poet::OdraPoet;

/// Generates the code for the given Odra module.
pub fn generate_code<T>(entity: T) -> proc_macro2::TokenStream
where
    T: OdraPoet
{
    <T as OdraPoet>::Poet::from(entity).generate_code()
}
