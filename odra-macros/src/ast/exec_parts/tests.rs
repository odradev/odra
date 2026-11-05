use super::*;
use crate::test_utils::{self, mock};

#[test]
fn test_parts() {
    let module = mock::module_impl();
    let actual = ModuleExecPartsItem::try_from(&module).unwrap();

    let expected = quote::quote! {
        #[allow(missing_docs)]
        mod __erc20_exec_parts {
            use super::*;
            use odra::prelude::*;

            #[inline]
            pub fn execute_init(__odra_env: odra::ContractEnv) {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                let total_supply = __odra_exec_env.get_named_arg::<Option<U256>>("total_supply");
                let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.init(total_supply);
                return __odra_result;
            }

            #[inline]
            pub fn execute_upgrade(__odra_env: odra::ContractEnv) {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                let total_supply = __odra_exec_env.get_named_arg::<Option<U256>>("total_supply");
                let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.upgrade(total_supply);
                return __odra_result;
            }

            #[inline]
            pub fn execute_total_supply(__odra_env: odra::ContractEnv) -> U256 {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.total_supply();
                return __odra_result;
            }

            #[inline]
            pub fn execute_pay_to_mint(__odra_env: odra::ContractEnv) {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                __odra_exec_env.handle_attached_value();
                let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.pay_to_mint();
                __odra_exec_env.clear_attached_value();
                return __odra_result;
            }

            #[inline]
            pub fn execute_approve(__odra_env: odra::ContractEnv) {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                __odra_exec_env.non_reentrant_before();
                let to = __odra_exec_env.get_named_arg::<Address>("to");
                let amount = __odra_exec_env.get_named_arg::<U256>("amount");
                let msg = __odra_exec_env.get_named_arg::<Maybe<String>>("msg");
                let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.approve(&to, &amount, msg);
                __odra_exec_env.non_reentrant_after();
                return __odra_result;
            }

            #[inline]
            pub fn execute_airdrop(__odra_env: odra::ContractEnv) {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                let to = __odra_exec_env.get_named_arg::<odra::prelude::vec::Vec<Address>>("to");
                let amount = __odra_exec_env.get_named_arg::<U256>("amount");
                let __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.airdrop(&to, &amount);
                return __odra_result;
            }

            #[inline]
            pub fn execute_swap(__odra_env: odra::ContractEnv) -> U256 {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                let to = __odra_exec_env.get_named_arg::<Address>("to");
                let amount = __odra_exec_env.get_named_arg::<U256>("amount");
                let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.swap(to, amount);
                return __odra_result;
            }
        }
    };

    test_utils::assert_eq(actual, expected);
}

#[test]
fn test_trait_impl_parts() {
    let module = mock::module_trait_impl();
    let actual = ModuleExecPartsItem::try_from(&module).unwrap();

    let expected = quote::quote! {
        #[allow(missing_docs)]
        mod __erc20_exec_parts {
            use super::*;
            use odra::prelude::*;

            #[inline]
            pub fn execute_total_supply(__odra_env: odra::ContractEnv) -> U256 {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.total_supply();
                return __odra_result;
            }

            #[inline]
            pub fn execute_set_total_supply(__odra_env: odra::ContractEnv) -> U256 {
                let __odra_env_rc = Rc::new(__odra_env);
                let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.set_total_supply();
                return __odra_result;
            }

            #[inline]
            pub fn execute_pay_to_mint(__odra_env: odra::ContractEnv) {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                __odra_exec_env.handle_attached_value();
                let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.pay_to_mint();
                __odra_exec_env.clear_attached_value();
                return __odra_result;
            }
        }
    };

    test_utils::assert_eq(actual, expected);
}

#[test]
fn test_delegated_parts() {
    let module = mock::module_delegation();
    let actual = ModuleExecPartsItem::try_from(&module).unwrap();

    let expected = quote::quote! {
        #[allow(missing_docs)]
        mod __erc20_exec_parts {
            use super::*;
            use odra::prelude::*;

            #[inline]
            pub fn execute_total_supply(__odra_env: odra::ContractEnv) -> U256 {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.total_supply();
                return __odra_result;
            }

            #[inline]
            pub fn execute_get_owner(__odra_env: odra::ContractEnv) -> Address {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.get_owner();
                return __odra_result;
            }

            #[inline]
            pub fn execute_set_owner(__odra_env: odra::ContractEnv) {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                let new_owner = __odra_exec_env.get_named_arg::<Address>("new_owner");
                let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.set_owner(new_owner);
                return __odra_result;
            }

            #[inline]
            pub fn execute_name(__odra_env: odra::ContractEnv) -> String {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.name();
                return __odra_result;
            }

            #[inline]
            pub fn execute_symbol(__odra_env: odra::ContractEnv) -> String {
                let __odra_env_rc = Rc::new(__odra_env);
                let __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                let __odra_result = __odra_contract.symbol();
                return __odra_result;
            }
        }
    };

    test_utils::assert_eq(actual, expected);
}

/// The same emitter built for the factory of the module.
mod factory {
    use super::*;

    #[test]
    fn test_parts() {
        let module = mock::module_factory_impl();
        let actual = ModuleExecPartsItem::try_new(&module, Target::Factory).unwrap();

        let expected = quote::quote! {
            #[allow(missing_docs)]
            mod __erc20_factory_exec_parts {
                use super::*;
                use odra::prelude::*;

                #[inline]
                pub fn execute_init(__odra_env: odra::ContractEnv) {
                    let __odra_env_rc = Rc::new(__odra_env);
                    let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                    let value = __odra_exec_env.get_named_arg::<u32>("value");
                    let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                    let __odra_result = __odra_contract.init(value);
                    return __odra_result;
                }

                #[inline]
                pub fn execute_total_supply(__odra_env: odra::ContractEnv) -> U256 {
                    let __odra_env_rc = Rc::new(__odra_env);
                    let __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                    let __odra_result = __odra_contract.total_supply();
                    return __odra_result;
                }

                #[inline]
                pub fn execute_pay_to_mint(__odra_env: odra::ContractEnv) {
                    let __odra_env_rc = Rc::new(__odra_env);
                    let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                    __odra_exec_env.handle_attached_value();
                    let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                    let __odra_result = __odra_contract.pay_to_mint();
                    __odra_exec_env.clear_attached_value();
                    return __odra_result;
                }

                #[inline]
                pub fn execute_approve(__odra_env: odra::ContractEnv) {
                    let __odra_env_rc = Rc::new(__odra_env);
                    let __odra_exec_env = odra::ExecutionEnv::new(__odra_env_rc.clone());
                    __odra_exec_env.non_reentrant_before();
                    let to = __odra_exec_env.get_named_arg::<Address>("to");
                    let amount = __odra_exec_env.get_named_arg::<U256>("amount");
                    let msg = __odra_exec_env.get_named_arg::<Maybe<String>>("msg");
                    let mut __odra_contract = <Erc20 as Module>::new(__odra_env_rc);
                    let __odra_result = __odra_contract.approve(&to, &amount, msg);
                    __odra_exec_env.non_reentrant_after();
                    return __odra_result;
                }
            }
        };

        test_utils::assert_eq(actual, expected);
    }
}
