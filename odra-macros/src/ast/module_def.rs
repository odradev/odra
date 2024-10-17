use crate::ir::ModuleStructIR;
use crate::utils;

const MAX_FIELDS: usize = 15;

#[derive(syn_derive::ToTokens)]
pub struct ModuleDefItem {
    item_struct: syn::ItemStruct
}

impl TryFrom<&'_ ModuleStructIR> for ModuleDefItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
        let mut item_struct = ir.self_code().clone();
        let env_field: syn::Field = utils::misc::field(
            &utils::ident::underscored_env(),
            &utils::ty::rc_contract_env()
        );

        if item_struct.fields.len() > MAX_FIELDS {
            return Err(syn::Error::new_spanned(
                item_struct.fields,
                format!("The number of fields in a module definition must be less than or equal to {}", MAX_FIELDS)
            ));
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
                __env: Rc<odra::ContractEnv>
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
                __env: Rc<odra::ContractEnv>
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
