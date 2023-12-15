use crate::ir::ModuleStructIR;
use crate::utils;

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
    use crate::test_utils::{assert_eq, mock_empty_module_definition, mock_module_definition};

    #[test]
    fn test_module_def_item() {
        let ir = mock_module_definition();
        let def = ModuleDefItem::try_from(&ir).unwrap();
        let expected = quote::quote! {
            pub struct CounterPack {
                counter0: ModuleWrapper<Counter>,
                counter1: ModuleWrapper<Counter>,
                counter2: ModuleWrapper<Counter>,
                counters: Variable<u32>,
                counters_map: Mapping<u8, Counter>,
                __env: Rc<odra::ContractEnv>
            }
        };

        assert_eq(def, expected);
    }

    #[test]
    fn empty_module() {
        let ir = mock_empty_module_definition();
        let def = ModuleDefItem::try_from(&ir).unwrap();
        let expected = quote::quote! {
            pub struct CounterPack {
                __env: Rc<odra::ContractEnv>
            }
        };
        assert_eq(def, expected);
    }
}
