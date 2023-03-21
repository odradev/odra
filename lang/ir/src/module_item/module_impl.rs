use std::convert::TryFrom;

use proc_macro2::{Ident, TokenStream};

use super::impl_item::ImplItem;

/// Odra module implementation block.
///
/// # Examples
/// ```
/// # <odra_ir::module::ModuleImpl as TryFrom<syn::ItemImpl>>::try_from(syn::parse_quote! {
/// impl MyModule {
///     #[odra(init)]
///     #[other_attribute]
///     pub fn set_initial_value(&self, value: u32) {
///         // initialization logic goes here
///     }
///
///     pub fn set_value(&self, value: u32) {
///         // logic goes here
///     }
/// }
/// # }).unwrap();
/// ```
pub struct ModuleImpl {
    delegation_stmts: Vec<DelegationStatement>,
    impl_items: Vec<ImplItem>,
    ident: Ident,
    is_trait_implementation: bool
}

mod kw {
    syn::custom_keyword!(to);
}

#[derive(Debug, Clone)]
pub struct Delegate {
    pub stmts: Vec<DelegationStatement>
}

#[derive(Debug, Clone)]
pub struct DelegationStatement {
    pub delegate_to: syn::ExprField,
    pub delegation_block: DelegationBlock
}

#[derive(Debug, Clone)]
pub struct DelegationBlock {
    pub brace_token: syn::token::Brace,
    pub functions: Vec<DelegatedFunction>
}

#[derive(Debug, Clone)]
pub struct DelegatedFunction {
    pub attrs: Vec<syn::Attribute>,
    pub visibility: syn::Visibility,
    pub fn_item: syn::TraitItemMethod
}

impl syn::parse::Parse for Delegate {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut stmts = vec![];
        while !input.is_empty() {
            stmts.push(input.parse::<DelegationStatement>()?);
        }
        Ok(Self {
            stmts
        })
    }
}

impl syn::parse::Parse for DelegationStatement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::to>()?;

        let delegate_to = input.parse::<syn::ExprField>()?;
        let delegation_block = input.parse::<DelegationBlock>()?;
        Ok(Self {
            delegate_to,
            delegation_block
        })
    }
}

impl syn::parse::Parse for DelegationBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let brace_token = syn::braced!(content in input);
        let mut functions = vec![];
        while !content.is_empty() {
            functions.push(content.parse::<DelegatedFunction>()?);
        }
        Ok(Self {
            brace_token,
            functions
        })
    }
}

impl syn::parse::Parse for DelegatedFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let visibility = input.parse::<syn::Visibility>()?;
        let fn_item = input.parse::<syn::TraitItemMethod>()?;
        Ok(Self {
            attrs,
            visibility,
            fn_item
        })
    }
}

impl ModuleImpl {
    pub fn impl_items(&self) -> &[ImplItem] {
        self.impl_items.as_ref()
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn methods(&self) -> Vec<&ImplItem> {
        self.impl_items
            .iter()
            .filter(|i| matches!(i, ImplItem::Method(_) | ImplItem::Constructor(_)))
            .collect::<Vec<_>>()
    }

    pub fn public_methods(&self) -> Vec<&ImplItem> {
        self.impl_items
            .iter()
            .filter(|item| match item {
                ImplItem::Constructor(_) => true,
                ImplItem::Method(m) => self.is_trait_implementation || m.is_public(),
                ImplItem::Other(_) => false,
                ImplItem::DelegatedMethod(m) => todo!(),
            })
            .collect::<Vec<_>>()
    }

    pub fn is_trait_implementation(&self) -> bool {
        self.is_trait_implementation
    }

    pub fn delegation_stmts(&self) -> &[DelegationStatement] {
        self.delegation_stmts.as_ref()
    }
}

impl TryFrom<syn::ItemImpl> for ModuleImpl {
    type Error = syn::Error;

    fn try_from(item_impl: syn::ItemImpl) -> Result<Self, Self::Error> {
        let is_trait_implementation = item_impl.trait_.is_some();
        let path = match &*item_impl.self_ty {
            syn::Type::Path(path) => path,
            _ => todo!()
        };
        let contract_ident = path.path.segments.last().unwrap().clone().ident;

        let delegation_stmts = item_impl
            .items
            .clone()
            .into_iter()
            .filter_map(|item| match item {
                syn::ImplItem::Macro(macro_item) => Some(macro_item),
                _ => None
            })
            .map(|macro_item| {
                syn::parse2::<Delegate>(TokenStream::from(macro_item.mac.tokens))
            })
            .collect::<Result<Vec<_>, syn::Error>>()?;

        let delegation_stmts = delegation_stmts.into_iter().map(|d| d.stmts).flatten().collect::<Vec<_>>();

        let items = item_impl
            .items
            .into_iter()
            .filter(|item| matches!(item, syn::ImplItem::Method(_)))
            .map(<ImplItem as TryFrom<_>>::try_from)
            .collect::<Result<Vec<_>, syn::Error>>()?;

        Ok(Self {
            impl_items: items,
            ident: contract_ident,
            is_trait_implementation,
            delegation_stmts
        })
    }
}

#[cfg(test)]
mod test {
    use super::ModuleImpl;

    #[test]
    fn impl_items_filtering() {
        let item_impl: syn::ItemImpl = syn::parse_quote! {
            impl Contract {

                #[odra(init)]
                pub fn constructor() {}

                pub(crate) fn crate_public_fn() {}

                pub fn public_fn() {}

                fn private_fn() {}

                delegate! {
                    to self.a {
                        pub fn public_fn_del(&self);
                        pub fn private_fn_del(&self);
                    }
                }
            }
        };
        let module_impl = ModuleImpl::try_from(item_impl).unwrap();

        assert_eq!(module_impl.methods().len(), 4);
        assert_eq!(module_impl.public_methods().len(), 2);
        // assert_eq!(module_impl.impl_items().len(), 5);
        assert_eq!(module_impl.delegation_stmts().len(), 1);
    }
}
