use quote::{ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::{
    ir::{EnumeratedTypedField, ModuleStructIR},
    utils::{self, expr::IntoExpr}
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

impl TryFrom<&'_ ModuleStructIR> for ModuleModItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
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

impl TryFrom<&'_ ModuleStructIR> for ModuleImplItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
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

impl TryFrom<&'_ ModuleStructIR> for NewModuleFnItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
        let ty_contract_env = utils::ty::rc_contract_env();
        let env = utils::ident::env();
        let fields = ir.typed_fields()?;
        Ok(Self {
            sig: parse_quote!(fn new(#env: #ty_contract_env) -> Self),
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
            field_expr: utils::expr::module_component_instance(
                &field.ty,
                &utils::ident::env(),
                field.idx
            ),
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
    values: syn::punctuated::Punctuated<ValueInitItem, syn::Token![,]>
}

impl TryFrom<&'_ ModuleStructIR> for ModuleInstanceItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
        let ident_underscored_env = utils::ident::underscored_env();
        let ident_env = utils::ident::env();
        let env_init = ValueInitItem::with_init(ident_underscored_env, ident_env.into_expr());

        Ok(Self {
            self_token: Default::default(),
            braces: Default::default(),
            values: ir
                .field_names()?
                .into_iter()
                .map(ValueInitItem::new)
                .chain(vec![env_init])
                .collect()
        })
    }
}

struct EnvFnItem;

impl ToTokens for EnvFnItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty_contract_env = utils::ty::rc_contract_env();
        let m_env = utils::member::underscored_env();

        tokens.extend(quote::quote!(
            fn env(&self) -> #ty_contract_env {
                #m_env.clone()
            }
        ))
    }
}

#[derive(syn_derive::ToTokens)]
struct ValueInitItem {
    ident: syn::Ident,
    colon_token: Option<syn::Token![:]>,
    init_expr: Option<syn::Expr>
}

impl ValueInitItem {
    fn new(ident: syn::Ident) -> Self {
        Self {
            ident,
            colon_token: None,
            init_expr: None
        }
    }

    fn with_init(ident: syn::Ident, init_expr: syn::Expr) -> Self {
        Self {
            ident,
            colon_token: Some(Default::default()),
            init_expr: Some(init_expr)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils;
    use quote::quote;

    use super::ModuleModItem;

    #[test]
    fn empty_module() {
        let module = test_utils::mock::empty_module_definition();
        let expected = quote!(
            mod __counter_pack_module {
                use super::*;

                impl odra::module::Module for CounterPack {
                    fn new(env: odra::prelude::Rc<odra::ContractEnv>) -> Self {
                        Self { __env: env }
                    }

                    fn env(&self) -> odra::prelude::Rc<odra::ContractEnv> {
                        self.__env.clone()
                    }
                }
            }
        );
        let actual = ModuleModItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }

    #[test]
    fn counter_pack() {
        let module = test_utils::mock::module_definition();
        let expected = quote!(
            mod __counter_pack_module {
                use super::*;

                impl odra::module::Module for CounterPack {
                    fn new(env: odra::prelude::Rc<odra::ContractEnv>) -> Self {
                        let counter0 =
                            <ModuleWrapper<Counter> as odra::module::ModuleComponent>::instance(
                                odra::prelude::Rc::clone(&env),
                                0u8
                            );
                        let counter1 =
                            <ModuleWrapper<Counter> as odra::module::ModuleComponent>::instance(
                                odra::prelude::Rc::clone(&env),
                                1u8
                            );
                        let counter2 =
                            <ModuleWrapper<Counter> as odra::module::ModuleComponent>::instance(
                                odra::prelude::Rc::clone(&env),
                                2u8
                            );
                        let counters = <Variable<u32> as odra::module::ModuleComponent>::instance(
                            odra::prelude::Rc::clone(&env),
                            3u8
                        );
                        let counters_map =
                            <Mapping<u8, Counter> as odra::module::ModuleComponent>::instance(
                                odra::prelude::Rc::clone(&env),
                                4u8
                            );
                        Self {
                            counter0,
                            counter1,
                            counter2,
                            counters,
                            counters_map,
                            __env: env
                        }
                    }

                    fn env(&self) -> odra::prelude::Rc<odra::ContractEnv> {
                        self.__env.clone()
                    }
                }
            }
        );
        let actual = ModuleModItem::try_from(&module).unwrap();
        test_utils::assert_eq(actual, expected);
    }
}
