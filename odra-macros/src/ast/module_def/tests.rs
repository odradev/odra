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
