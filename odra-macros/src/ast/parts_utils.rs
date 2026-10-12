use quote::{quote, ToTokens};

#[derive(Default)]
pub struct UsePreludeItem;

impl ToTokens for UsePreludeItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(
            use odra::prelude::*;
        ));
    }
}

#[derive(Default)]
pub struct UseSuperItem;

impl ToTokens for UseSuperItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(
            use super::*;
        ));
    }
}

/// Wraps an item gated by the `odra_module` cfg in an anonymous const block that allows the
/// `unexpected_cfgs` lint.
///
/// `odra_module` is set (and declared to rustc's check-cfg) by `odra_build::build()`. A crate
/// that uses Odra modules without that build script never sets it, so the gated item is
/// compiled out, but rustc still reports the cfg name as unexpected at the generated use site.
/// A lint attribute on the gated item itself does not silence it - the cfg is evaluated before
/// the item exists - so the item is nested in `const _: () = { .. };`, which keeps it at the same
/// module level (`use super::*` still resolves to the user's module and `#[no_mangle]` symbols
/// are still exported).
pub struct OdraModuleCfgItem<T>(T);

impl<'a, T> TryFrom<&'a crate::ir::ModuleImplIR> for OdraModuleCfgItem<T>
where
    T: TryFrom<&'a crate::ir::ModuleImplIR, Error = syn::Error>
{
    type Error = syn::Error;

    fn try_from(ir: &'a crate::ir::ModuleImplIR) -> Result<Self, Self::Error> {
        T::try_from(ir).map(Self)
    }
}

impl<'a, T> TryFrom<&'a crate::ir::ModuleStructIR> for OdraModuleCfgItem<T>
where
    T: TryFrom<&'a crate::ir::ModuleStructIR, Error = syn::Error>
{
    type Error = syn::Error;

    fn try_from(ir: &'a crate::ir::ModuleStructIR) -> Result<Self, Self::Error> {
        T::try_from(ir).map(Self)
    }
}

impl<T: ToTokens> ToTokens for OdraModuleCfgItem<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let item = &self.0;
        tokens.extend(quote!(
            #[allow(unexpected_cfgs)]
            const _: () = {
                #item
            };
        ));
    }
}
