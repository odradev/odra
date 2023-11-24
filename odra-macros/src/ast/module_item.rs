use quote::{ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::{
    ir::{EnumeratedTypedField, StructIR},
    utils
};

use super::parts_utils::UseSuperItem;

#[derive(syn_derive::ToTokens)]
pub struct ModuleModItem {
    mod_token: syn::token::Mod,
    mod_ident: syn::Ident,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    use_super: UseSuperItem,
    #[syn(in = braces)]
    item: ModuleImplItem
}

impl TryFrom<&'_ StructIR> for ModuleModItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ StructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            mod_token: Default::default(),
            mod_ident: ir.module_mod_ident(),
            use_super: UseSuperItem,
            braces: Default::default(),
            item: ir.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct ModuleImplItem {
    impl_token: syn::token::Impl,
    trait_path: syn::Type,
    for_token: syn::token::For,
    module_path: syn::Ident,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    new_fn: NewModuleFnItem,
    #[syn(in = braces)]
    env_fn: EnvFnItem
}

impl TryFrom<&'_ StructIR> for ModuleImplItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ StructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            impl_token: Default::default(),
            trait_path: utils::ty::module(),
            for_token: Default::default(),
            module_path: ir.module_ident(),
            braces: Default::default(),
            new_fn: ir.try_into()?,
            env_fn: EnvFnItem
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct NewModuleFnItem {
    sig: syn::Signature,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    #[to_tokens(|tokens, val| tokens.append_all(val))]
    fields: Vec<ModuleFieldItem>,
    #[syn(in = braces)]
    instance: ModuleInstanceItem
}

impl TryFrom<&'_ StructIR> for NewModuleFnItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ StructIR) -> Result<Self, Self::Error> {
        let ty_contract_env = utils::ty::contract_env();
        let env = utils::ident::env();
        let fields = ir.typed_fields()?;
        Ok(Self {
            sig: parse_quote!(fn new(#env: Rc<#ty_contract_env>) -> Self),
            braces: Default::default(),
            fields: fields.iter().map(Into::into).collect(),
            instance: ir.try_into()?
        })
    }
}

#[derive(syn_derive::ToTokens)]
struct ModuleFieldItem {
    let_token: syn::token::Let,
    ident: syn::Ident,
    assign_token: syn::token::Eq,
    field_expr: syn::Expr,
    semi_token: syn::token::Semi
}

impl From<&'_ EnumeratedTypedField> for ModuleFieldItem {
    fn from(field: &'_ EnumeratedTypedField) -> Self {
        Self {
            let_token: Default::default(),
            ident: field.ident.clone(),
            assign_token: Default::default(),
            field_expr: utils::expr::new_type(&field.ty, &utils::ident::env(), field.idx),
            semi_token: Default::default()
        }
    }
}

#[derive(syn_derive::ToTokens)]
struct ModuleInstanceItem {
    self_token: syn::token::SelfType,
    #[syn(braced)]
    braces: syn::token::Brace,
    #[syn(in = braces)]
    values: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]>
}

impl TryFrom<&'_ StructIR> for ModuleInstanceItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ StructIR) -> Result<Self, Self::Error> {
        Ok(Self {
            self_token: Default::default(),
            braces: Default::default(),
            values: ir.field_names()?.into_iter().collect()
        })
    }
}

struct EnvFnItem;

impl ToTokens for EnvFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty_contract_env = utils::ty::contract_env();
        let m_env = utils::member::env();

        tokens.extend(quote::quote!(
            fn env(&self) -> Rc<#ty_contract_env> {
                #m_env.clone()
            }
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils;
    use quote::quote;

    use super::ModuleModItem;

    #[test]
    fn counter_pack() {
        let module = test_utils::mock_module_definition();
        let expected = quote!(
            mod __counter_pack_module {
                use super::*;

                impl odra::module::Module for CounterPack {
                    fn new(env: Rc<odra::ContractEnv>) -> Self {
                        let counter0 = ModuleWrapper::new(Rc::clone(&env), 0u8);
                        let counter1 = ModuleWrapper::new(Rc::clone(&env), 1u8);
                        let counter2 = ModuleWrapper::new(Rc::clone(&env), 2u8);
                        let counters = Variable::new(Rc::clone(&env), 3u8);
                        let counters_map = Mapping::new(Rc::clone(&env), 4u8);
                        Self {
                            env,
                            counter0,
                            counter1,
                            counter2,
                            counters,
                            counters_map
                        }
                    }

                    fn env(&self) -> Rc<odra::ContractEnv> {
                        self.env.clone()
                    }
                }
            }
        );
        let actual = ModuleModItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
