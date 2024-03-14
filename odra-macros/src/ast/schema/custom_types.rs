use quote::ToTokens;

use crate::{
    ir::{FnIR, ModuleImplIR},
    utils::ty
};

pub struct SchemaCustomTypesItem {
    module_ident: syn::Ident,
    fns: Vec<FnIR>
}

impl ToTokens for SchemaCustomTypesItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let module_ident = &self.module_ident;
        let types = self
            .fns
            .iter()
            .flat_map(|f| {
                let args = f.typed_args();
                let mut types = args
                    .iter()
                    .map(|arg| ty::unreferenced_ty(&arg.ty))
                    .collect::<Vec<_>>();
                if let syn::ReturnType::Type(_, t) = f.return_type() {
                    types.push(*t);
                };
                types
            })
            .collect::<Vec<_>>();

        let types = types
            .iter()
            .map(|t| {
                quote::quote! {
                    <#t as odra::schema::SchemaCustomType>::custom_ty()
                }
            })
            .collect::<Vec<_>>();

        let item = quote::quote! {
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomTypes for #module_ident {
                fn schema_types() -> Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    vec![ #(#types),* ]
                }
            }
        };

        item.to_tokens(tokens);
    }
}

impl TryFrom<&ModuleImplIR> for SchemaCustomTypesItem {
    type Error = syn::Error;

    fn try_from(ir: &ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self {
            module_ident: ir.module_ident()?,
            fns: ir.functions()?
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[test]
    fn custom_types_works() {
        let ir = test_utils::mock::module_impl();
        let item = SchemaCustomTypesItem::try_from(&ir).unwrap();
        let expected = quote::quote!(
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaCustomTypes for Erc20 {
                fn schema_types() -> Vec<Option<odra::schema::casper_contract_schema::CustomType>> {
                    vec![
                        <Option<U256> as odra::schema::SchemaCustomType>::custom_ty(),
                        <U256 as odra::schema::SchemaCustomType>::custom_ty(),
                        <Address as odra::schema::SchemaCustomType>::custom_ty(),
                        <U256 as odra::schema::SchemaCustomType>::custom_ty(),
                        <Maybe<String> as odra::schema::SchemaCustomType>::custom_ty(),
                        <odra::prelude::vec::Vec<Address> as odra::schema::SchemaCustomType>::custom_ty(),
                        <U256 as odra::schema::SchemaCustomType>::custom_ty()
                    ]
                }
            }
        );

        test_utils::assert_eq(item, expected);
    }
}
