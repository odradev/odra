use crate::test_utils;
use quote::quote;

use super::ModuleModItem;

#[test]
fn empty_module() {
    let module = test_utils::mock::empty_module_definition();
    let expected = quote!(
        mod __counter_pack_module {
            use super::*;

            impl Module for CounterPack {
                fn new(__odra_env: odra::prelude::Rc<odra::ContractEnv>) -> Self {
                    Self { __odra_env }
                }

                fn env(&self) -> odra::prelude::Rc<odra::ContractEnv> {
                    self.__odra_env.clone()
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

            impl Module for CounterPack {
                fn new(__odra_env: odra::prelude::Rc<odra::ContractEnv>) -> Self {
                    let counter0 =
                        <SubModule<Counter> as odra::module::ModuleComponent>::instance(
                            odra::prelude::Rc::clone(&__odra_env),
                            1u8
                        );
                    let counter1 =
                        <SubModule<Counter> as odra::module::ModuleComponent>::instance(
                            odra::prelude::Rc::clone(&__odra_env),
                            2u8
                        );
                    let counter2 =
                        <SubModule<Counter> as odra::module::ModuleComponent>::instance(
                            odra::prelude::Rc::clone(&__odra_env),
                            3u8
                        );
                    let counters = <Var<u32> as odra::module::ModuleComponent>::instance(
                        odra::prelude::Rc::clone(&__odra_env),
                        4u8
                    );
                    let counters_map =
                        <Mapping<u8, Counter> as odra::module::ModuleComponent>::instance(
                            odra::prelude::Rc::clone(&__odra_env),
                            5u8
                        );
                    Self {
                        counter0,
                        counter1,
                        counter2,
                        counters,
                        counters_map,
                        __odra_env
                    }
                }

                fn env(&self) -> odra::prelude::Rc<odra::ContractEnv> {
                    self.__odra_env.clone()
                }
            }
        }
    );
    let actual = ModuleModItem::try_from(&module).unwrap();
    test_utils::assert_eq(actual, expected);
}
