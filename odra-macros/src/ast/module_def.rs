use crate::ir::{errors, ModuleStructIR};
use crate::utils;

const MAX_FIELDS: usize = 255;

#[derive(syn_derive::ToTokens)]
pub struct ModuleDefItem {
    item_struct: syn::ItemStruct
}

impl TryFrom<&'_ ModuleStructIR> for ModuleDefItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
        let mut item_struct = ir.self_code().clone();
        let env_field: syn::Field = utils::misc::field(
            &utils::ident::prefixed_env(),
            &utils::ty::rc_contract_env()
        );

        if item_struct.fields.len() > MAX_FIELDS {
            return Err(errors::module_too_many_fields(&item_struct.fields, MAX_FIELDS));
        }

        let fields = item_struct
            .fields
            .into_iter()
            .chain(vec![env_field])
            .collect::<syn::punctuated::Punctuated<_, _>>();

        item_struct.fields = syn::Fields::Named(syn::FieldsNamed {
            brace_token: Default::default(),
            named: fields
        });

        Ok(Self { item_struct })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{assert_eq, mock};

    #[test]
    fn test_module_def_item() {
        let ir = mock::module_definition();
        let def = ModuleDefItem::try_from(&ir).unwrap();
        let expected = quote::quote! {
            pub struct CounterPack {
                counter0: SubModule<Counter>,
                counter1: SubModule<Counter>,
                counter2: SubModule<Counter>,
                counters: Var<u32>,
                counters_map: Mapping<u8, Counter>,
                __odra_env: odra::prelude::Rc<odra::ContractEnv>
            }
        };

        assert_eq(def, expected);
    }

    #[test]
    fn empty_module() {
        let ir = mock::empty_module_definition();
        let def = ModuleDefItem::try_from(&ir).unwrap();
        let expected = quote::quote! {
            pub struct CounterPack {
                __odra_env: odra::prelude::Rc<odra::ContractEnv>
            }
        };
        assert_eq(def, expected);
    }

    #[test]
    fn test_invalid_module_definition() {
        let ir = mock::invalid_module_definition();
        let def = ModuleDefItem::try_from(&ir);
        assert!(def.is_err());
    }
}
