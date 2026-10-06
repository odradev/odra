use crate::utils;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

/// `#[odra::ref_helpers] impl Xxx { .. }` - the same inherent impl for `XxxContractRef` and
/// `XxxHostRef`.
///
/// The impl block is not emitted for `Xxx` itself; its items are copied verbatim, so a helper
/// can call any function both refs have - the entry points.
pub struct RefHelpersItem {
    contract_ref_impl: syn::ItemImpl,
    host_ref_impl: syn::ItemImpl
}

impl TryFrom<&'_ TokenStream> for RefHelpersItem {
    type Error = syn::Error;

    fn try_from(item: &'_ TokenStream) -> Result<Self, Self::Error> {
        let item_impl = syn::parse2::<syn::ItemImpl>(item.clone()).map_err(|_| {
            syn::Error::new(
                item.span(),
                "#[odra::ref_helpers] can be applied to an inherent impl block only"
            )
        })?;
        if let Some((_, path, _)) = &item_impl.trait_ {
            return Err(syn::Error::new(
                path.span(),
                "#[odra::ref_helpers] can be applied to an inherent impl block only, not a trait impl"
            ));
        }
        if !item_impl.generics.params.is_empty() {
            return Err(syn::Error::new(
                item_impl.generics.span(),
                "#[odra::ref_helpers] does not support generics on the impl block"
            ));
        }
        let path = match item_impl.self_ty.as_ref() {
            syn::Type::Path(syn::TypePath { qself: None, path }) => path,
            ty => {
                return Err(syn::Error::new(
                    ty.span(),
                    "#[odra::ref_helpers] expects a module or an external contract name"
                ))
            }
        };
        let last = path.segments.last().expect("a path has at least one segment");
        if !last.arguments.is_none() {
            return Err(syn::Error::new(
                last.arguments.span(),
                "#[odra::ref_helpers] does not support generic arguments"
            ));
        }

        let with_suffix = |suffix: &str| -> syn::Type {
            let mut path = path.clone();
            let last = path.segments.last_mut().expect("checked above");
            last.ident = syn::Ident::new(&format!("{}{}", last.ident, suffix), last.ident.span());
            syn::Type::Path(syn::TypePath { qself: None, path })
        };
        let contract_ref_impl = syn::ItemImpl {
            self_ty: Box::new(with_suffix("ContractRef")),
            ..item_impl.clone()
        };
        let mut host_ref_impl = syn::ItemImpl {
            self_ty: Box::new(with_suffix("HostRef")),
            ..item_impl
        };
        host_ref_impl.attrs.push(utils::attr::not_wasm32());
        Ok(Self {
            contract_ref_impl,
            host_ref_impl
        })
    }
}

impl quote::ToTokens for RefHelpersItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.contract_ref_impl.to_tokens(tokens);
        self.host_ref_impl.to_tokens(tokens);
    }
}

#[cfg(test)]
mod test {
    use super::RefHelpersItem;
    use crate::test_utils;

    #[test]
    fn ref_helpers() {
        let item = quote::quote! {
            /// Helpers.
            impl crate::token::Token {
                /// Balance of the owner.
                pub fn owner_balance(&self) -> U256 {
                    self.balance_of(self.owner())
                }
            }
        };
        let item = RefHelpersItem::try_from(&item).unwrap();
        let expected = quote::quote! {
            /// Helpers.
            impl crate::token::TokenContractRef {
                /// Balance of the owner.
                pub fn owner_balance(&self) -> U256 {
                    self.balance_of(self.owner())
                }
            }

            /// Helpers.
            #[cfg(not(target_arch = "wasm32"))]
            impl crate::token::TokenHostRef {
                /// Balance of the owner.
                pub fn owner_balance(&self) -> U256 {
                    self.balance_of(self.owner())
                }
            }
        };
        test_utils::assert_eq(item, expected);
    }

    #[test]
    fn rejects_trait_impl() {
        let item = quote::quote!(impl Foo for Token {});
        assert!(RefHelpersItem::try_from(&item).is_err());
    }

    #[test]
    fn rejects_generics() {
        let item = quote::quote!(impl<T> Token<T> {});
        assert!(RefHelpersItem::try_from(&item).is_err());
        let item = quote::quote!(impl Token<u32> {});
        assert!(RefHelpersItem::try_from(&item).is_err());
    }

    #[test]
    fn rejects_non_impl() {
        let item = quote::quote!(
            struct Token;
        );
        assert!(RefHelpersItem::try_from(&item).is_err());
    }
}
