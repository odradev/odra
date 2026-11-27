mod validation {
    use crate::ir::{function::ImplContext, FnIR, ModuleImplIR};

    #[test]
    fn test_not_reserved_fn_arg() {
        let code: syn::ImplItemFn = syn::parse_quote!(
            pub fn abc(&self, arg_name: u64) {}
        );
        let result = FnIR::try_new_impl(code, ImplContext::default());
        assert!(result.is_ok());
    }

    // The rejected arguments, attributes and names (reserved arguments, module arguments on an
    // impl or a trait, `payable` on `init`...) are covered by the compile-fail tests of the `odra`
    // crate (`tests/macros/fail`), which check the message and its span.

    #[test]
    fn test_module_args_factory_or_none_on_impl_are_allowed() {
        let item = quote::quote!(
            impl Token {
                pub fn transfer(&mut self) {}
            }
        );
        let attr = quote::quote!(factory = on);
        assert!(ModuleImplIR::try_from((&attr, &item)).is_ok());
        let attr = quote::quote!();
        assert!(ModuleImplIR::try_from((&attr, &item)).is_ok());
    }

    #[test]
    fn test_module_without_args_on_trait_is_allowed() {
        let item = quote::quote!(
            pub trait Token {
                fn transfer(&mut self);
            }
        );
        let attr = quote::quote!();
        assert!(ModuleImplIR::try_from((&attr, &item)).is_ok());
    }

    #[test]
    fn test_payable_regular_fn_is_allowed() {
        let code: syn::ImplItemFn = syn::parse_quote!(
            #[odra(payable)]
            pub fn deposit(&mut self) {}
        );
        assert!(FnIR::try_new_impl(code, ImplContext::default()).is_ok());
    }

    #[test]
    fn test_non_odra_attribute_on_constructor_is_allowed() {
        let code: syn::ImplItemFn = syn::parse_quote!(
            /// Initializes the contract.
            #[allow(dead_code)]
            pub fn init(&mut self) {}
        );
        assert!(FnIR::try_new_impl(code, ImplContext::default()).is_ok());
    }
}

mod rejected_items {
    use crate::test_utils::mock;

    #[test]
    fn a_module_with_more_fields_than_the_storage_index_holds() {
        assert!(mock::invalid_module_definition().is_err());
    }

    #[test]
    fn a_union_is_not_an_odra_type() {
        assert!(mock::custom_union().is_err());
    }
}

mod offchain {
    use crate::ir::{FnIR, ModuleImplIR};
    use quote::quote;

    fn parse(module: proc_macro2::TokenStream) -> syn::Result<ModuleImplIR> {
        ModuleImplIR::try_from((&quote!(), &module))
    }

    #[test]
    fn offchain_functions_are_split_from_the_entry_points() {
        let module = parse(quote! {
            impl Token {
                pub fn balance_of(&self, owner: &Address) -> U256 { U256::zero() }
                #[odra(offchain)]
                pub fn balances(&self, owners: Vec<Address>) -> Vec<U256> { vec![] }
            }
        })
        .unwrap();
        let names = |fns: Vec<&FnIR>| fns.into_iter().map(FnIR::name_str).collect::<Vec<_>>();
        assert_eq!(
            names(module.functions().iter().collect()),
            vec!["balance_of", "balances"]
        );
        assert_eq!(names(module.onchain_functions()), vec!["balance_of"]);
        assert_eq!(names(module.offchain_functions()), vec!["balances"]);
    }
}

/// The host ref gets `try_x` for every entry point `x`, and `x_no_ret` and `try_x_no_ret` for one
/// that takes `&mut self` and returns a value: an entry point named like one of them is rejected
/// on its name, instead of rustc reporting a function defined twice on the macro call.
mod host_ref_name_collisions {
    use crate::ir::ModuleImplIR;
    use quote::quote;

    fn parse(item: proc_macro2::TokenStream) -> syn::Result<ModuleImplIR> {
        ModuleImplIR::try_from((&quote!(), &item))
    }

    fn messages(item: proc_macro2::TokenStream) -> Vec<String> {
        match parse(item) {
            Ok(_) => vec![],
            Err(err) => err.into_iter().map(|e| e.to_string()).collect()
        }
    }

    #[test]
    fn an_entry_point_named_like_the_try_version_of_another_is_rejected() {
        let expected = vec![
            "`try_transfer` is the name of the `try_` version of `transfer` the host ref \
             generates; rename one of the functions",
        ];
        assert_eq!(
            messages(quote! {
                impl Token {
                    pub fn transfer(&mut self, amount: U256) {}
                    pub fn try_transfer(&mut self, amount: U256) {}
                }
            }),
            expected
        );
        // Whatever the order, the function whose name is taken is reported.
        assert_eq!(
            messages(quote! {
                impl Token {
                    pub fn try_transfer(&mut self, amount: U256) {}
                    pub fn transfer(&mut self, amount: U256) {}
                }
            }),
            expected
        );
    }

    #[test]
    fn an_entry_point_named_like_a_no_ret_version_is_rejected() {
        assert_eq!(
            messages(quote! {
                impl Token {
                    pub fn swap(&mut self, amount: U256) -> U256 { amount }
                    pub fn swap_no_ret(&self) {}
                    pub fn try_swap_no_ret(&self) {}
                }
            }),
            vec![
                "`swap_no_ret` is the name of the `_no_ret` version of `swap` the host ref \
                 generates (`swap` takes `&mut self` and returns a value); rename one of the \
                 functions",
                "`try_swap_no_ret` is the name of the `try_.._no_ret` version of `swap` the host \
                 ref generates (`swap` takes `&mut self` and returns a value); rename one of the \
                 functions",
                // A different pair: `try_swap_no_ret` is also the `try_` version of `swap_no_ret`.
                "`try_swap_no_ret` is the name of the `try_` version of `swap_no_ret` the host ref \
                 generates; rename one of the functions"
            ]
        );
    }

    #[test]
    fn a_no_ret_name_is_free_when_no_no_ret_version_is_generated() {
        // `swap` takes `&self`, or returns nothing: the host ref has no `swap_no_ret`.
        assert!(parse(quote! {
            impl Token {
                pub fn swap(&self, amount: U256) -> U256 { amount }
                pub fn swap_no_ret(&mut self) {}
                pub fn get(&mut self) {}
                pub fn get_no_ret(&mut self) {}
            }
        })
        .is_ok());
    }

    #[test]
    fn two_generated_names_that_meet_are_rejected_on_the_later_function() {
        // `try_swap` gets `try_swap_no_ret` as its `_no_ret` version, `swap_no_ret` gets it as
        // its `try_` version.
        assert_eq!(
            messages(quote! {
                impl Token {
                    pub fn try_swap(&mut self) -> U256 { U256::zero() }
                    pub fn swap_no_ret(&self) {}
                }
            }),
            vec![
                "the host ref generates `try_swap_no_ret` twice: as the `_no_ret` version of \
                 `try_swap` and as the `try_` version of `swap_no_ret`; rename one of the \
                 functions"
            ]
        );
    }

    #[test]
    fn one_error_per_pair_of_functions() {
        // `try_transfer` is the `try_` version of `transfer`, and both get
        // `try_transfer_no_ret`: reported once.
        assert_eq!(
            messages(quote! {
                impl Token {
                    pub fn transfer(&mut self) -> U256 { U256::zero() }
                    pub fn try_transfer(&mut self) -> U256 { U256::zero() }
                }
            })
            .len(),
            1
        );
    }

    #[test]
    fn delegated_functions_are_checked_too() {
        assert_eq!(
            messages(quote! {
                impl Token {
                    pub fn try_name(&self) -> String { String::new() }
                    delegate! {
                        to self.metadata {
                            fn name(&self) -> String;
                        }
                    }
                }
            }),
            vec![
                "`try_name` is the name of the `try_` version of `name` the host ref generates; \
                 rename one of the functions"
            ]
        );
    }

    #[test]
    fn trait_impls_and_external_contracts_are_checked_too() {
        let message = "`try_transfer` is the name of the `try_` version of `transfer` the host \
                       ref generates; rename one of the functions";
        assert_eq!(
            messages(quote! {
                impl Transfer for Token {
                    fn transfer(&mut self) {}
                    fn try_transfer(&mut self) {}
                }
            }),
            vec![message]
        );
        assert_eq!(
            messages(quote! {
                pub trait Token {
                    fn transfer(&mut self);
                    fn try_transfer(&mut self);
                }
            }),
            vec![message]
        );
    }

    #[test]
    fn private_functions_are_not_entry_points() {
        // A private function is not in the host ref, so its name collides with nothing.
        assert!(parse(quote! {
            impl Token {
                pub fn transfer(&mut self) {}
                fn try_transfer(&mut self) {}
            }
        })
        .is_ok());
    }
}
