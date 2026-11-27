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

mod module_struct {
    use crate::ir::ModuleStructIR;
    use quote::quote;

    fn module_with_fields(count: usize) -> syn::Result<ModuleStructIR> {
        let fields = (0..count).map(|i| {
            let name = quote::format_ident!("field{}", i);
            quote!(#name: Var<u32>,)
        });
        let module = quote!(
            pub struct Module {
                #(#fields)*
            }
        );
        ModuleStructIR::try_from((&quote!(), &module))
    }

    #[test]
    fn a_module_with_as_many_fields_as_the_storage_index_holds() {
        let module = module_with_fields(255).unwrap();
        let indexes = module.typed_fields().iter().map(|f| f.idx);
        assert!(indexes.eq(1..=255));
    }

    #[test]
    fn factory_is_switched_on_and_off() {
        let module = quote!(
            pub struct Token;
        );
        let factory = |attr| {
            ModuleStructIR::try_from((&attr, &module))
                .unwrap()
                .is_factory()
        };
        assert!(factory(quote!(factory = on)));
        assert!(!factory(quote!(factory = off)));
        assert!(!factory(quote!()));
    }
}

/// The checks of the macros' input, each against one input it rejects: these pin which input
/// is rejected; the message and the span of each error are pinned by the compile-fail tests of
/// the `odra` crate (`tests/macros/fail`).
mod rejected_input {
    use crate::ir::{function::ImplContext, FnIR, ModuleImplIR, ModuleStructIR, TypeIR};
    use quote::quote;

    #[track_caller]
    fn assert_rejected<T>(result: syn::Result<T>, message: &str) {
        match result {
            Ok(_) => panic!("accepted; expected an error saying: {message}"),
            Err(error) => {
                let actual = error.into_iter().map(|e| e.to_string()).collect::<Vec<_>>();
                assert!(
                    actual.iter().any(|e| e.contains(message)),
                    "expected an error saying: {message}\ngot: {actual:?}"
                );
            }
        }
    }

    fn function(code: proc_macro2::TokenStream) -> syn::Result<FnIR> {
        FnIR::try_new_impl(syn::parse2(code).unwrap(), ImplContext::default())
    }

    #[test]
    fn a_reserved_function_name() {
        assert_rejected(
            function(quote!(
                pub fn env(&self) {}
            )),
            "`env` is a reserved entry point name"
        );
    }

    #[test]
    fn a_reserved_argument_name() {
        assert_rejected(
            function(quote!(
                pub fn pay(&self, gas: u64) {}
            )),
            "`gas` is a reserved argument name"
        );
    }

    #[test]
    fn a_mut_ref_argument() {
        assert_rejected(
            function(quote!(
                pub fn fill(&self, buf: &mut Vec<u8>) {}
            )),
            "`&mut` is not an argument type"
        );
    }

    #[test]
    fn a_function_without_receiver() {
        assert_rejected(
            function(quote!(
                pub fn total() -> u32 {
                    0
                }
            )),
            "`total` takes no `self`"
        );
    }

    #[test]
    fn an_odra_attribute_on_init_and_upgrade() {
        for name in ["init", "upgrade"] {
            let name = quote::format_ident!("{name}");
            assert_rejected(
                function(quote!(#[odra(non_reentrant)] pub fn #name(&mut self) {})),
                &format!("it is not applied when generating the `{name}` entrypoint")
            );
        }
    }

    #[test]
    fn an_offchain_function_taking_mut_self() {
        assert_rejected(
            function(quote!(
                #[odra(offchain)]
                pub fn sync(&mut self) {}
            )),
            "an offchain function runs on the host and cannot change the state"
        );
    }

    #[test]
    fn a_flag_given_twice() {
        assert_rejected(
            function(quote!(
                #[odra(payable, payable)]
                pub fn pay(&mut self) {}
            )),
            "duplicate odra attribute `payable`"
        );
        assert_rejected(
            function(quote!(
                #[odra(payable)]
                #[odra(payable)]
                pub fn pay(&mut self) {}
            )),
            "duplicate odra attribute `payable`"
        );
    }

    #[test]
    fn an_empty_odra_attribute() {
        assert_rejected(
            function(quote!(
                #[odra()]
                pub fn pay(&mut self) {}
            )),
            "empty odra attribute"
        );
    }

    #[test]
    fn an_error_variant_with_data() {
        let ty = TypeIR::try_from(&quote!(
            enum Error {
                Overflow(u32)
            }
        ))
        .unwrap();
        assert_rejected(ty.error_variants(), "`Overflow` carries data");
    }

    #[test]
    fn a_module_argument_given_twice() {
        let module = quote!(
            pub struct Token;
        );
        let attr = quote!(name = "A", name = "B");
        assert_rejected(
            ModuleStructIR::try_from((&attr, &module)),
            "`name` is given twice"
        );
    }

    #[test]
    fn an_impl_is_read() {
        // The context the function checks above run in: a plain module impl.
        let item = quote!(
            impl Token {
                pub fn transfer(&mut self) {}
            }
        );
        assert!(ModuleImplIR::try_from((&quote!(), &item)).is_ok());
    }
}
