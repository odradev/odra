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

mod receiver_mutability {
    use crate::ir::ModuleImplIR;
    use quote::quote;

    fn parse(module: proc_macro2::TokenStream) -> syn::Result<ModuleImplIR> {
        ModuleImplIR::try_from((&quote!(), &module))
    }

    /// `is_mut` follows the reference (`&mut`), not the binding (`mut self`).
    #[test]
    fn is_mut_follows_the_reference_not_the_binding() {
        let module = parse(quote! {
            impl Counter {
                pub fn get(&self) -> u32 { 0 }
                pub fn set(&mut self) {}
                pub fn typed_get(self: &Self) -> u32 { 0 }
                pub fn typed_set(self: &mut Self) {}
                pub fn mut_binding_get(mut self: &Self) -> u32 { 0 }
                pub fn mut_binding_set(mut self: &mut Self) {}
            }
        })
        .unwrap();
        let is_mut = module
            .functions()
            .iter()
            .map(|f| (f.name_str(), f.is_mut()))
            .collect::<Vec<_>>();
        assert_eq!(
            is_mut,
            vec![
                ("get".to_string(), false),
                ("set".to_string(), true),
                ("typed_get".to_string(), false),
                ("typed_set".to_string(), true),
                ("mut_binding_get".to_string(), false),
                ("mut_binding_set".to_string(), true)
            ]
        );
    }

    #[test]
    fn offchain_rejects_a_typed_mut_receiver() {
        let module = parse(quote! {
            impl Token {
                #[odra(offchain)]
                pub fn balances(self: &mut Self) -> u32 { 0 }
            }
        });
        assert!(module.is_err());
    }

    #[test]
    fn offchain_accepts_a_mut_binding_to_a_shared_reference() {
        let module = parse(quote! {
            impl Token {
                #[odra(offchain)]
                pub fn balances(mut self: &Self) -> u32 { 0 }
            }
        });
        assert!(module.is_ok());
    }
}
