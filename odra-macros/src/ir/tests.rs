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
    fn an_odra_attribute_on_a_private_function() {
        let item = quote!(
            impl Token {
                #[odra(payable)]
                fn helper(&self) {}
            }
        );
        assert_rejected(
            ModuleImplIR::try_from((&quote!(), &item)),
            "only an entry point can receive attached tokens"
        );
    }

    #[test]
    fn a_function_both_written_and_delegated() {
        let item = quote!(
            impl Token {
                pub fn owner(&self) -> Address {
                    self.ownable.owner()
                }
                delegate! {
                    to self.ownable {
                        fn owner(&self) -> Address;
                    }
                }
            }
        );
        assert_rejected(
            ModuleImplIR::try_from((&quote!(), &item)),
            "`owner` is defined twice in this impl"
        );
    }

    #[test]
    fn an_external_contract_function_with_a_body() {
        let item = quote!(
            pub trait Token {
                fn total(&self) -> U256 {
                    U256::zero()
                }
            }
        );
        assert_rejected(
            ModuleImplIR::try_from((&quote!(), &item)),
            "`total` has a body"
        );
    }

    #[test]
    fn an_external_contract_function_with_pub() {
        let item = quote!(
            pub trait Token {
                pub fn total(&self) -> U256;
            }
        );
        assert_rejected(
            ModuleImplIR::try_from((&quote!(), &item)),
            "a trait function is written without `pub`"
        );
    }

    fn module(
        attr: proc_macro2::TokenStream,
        item: proc_macro2::TokenStream
    ) -> syn::Result<ModuleImplIR> {
        ModuleImplIR::try_from((&attr, &item))
    }

    #[test]
    fn a_signature_that_is_not_a_plain_method() {
        assert_rejected(
            function(quote!(
                pub async fn run(&self) {}
            )),
            "`async` is not allowed on `run`"
        );
        assert_rejected(
            function(quote!(
                pub fn run<T>(&self, value: T) {}
            )),
            "`run` has generic parameters"
        );
        assert_rejected(
            function(quote!(
                pub fn run(self) {}
            )),
            "`self` is taken by value"
        );
    }

    #[test]
    fn a_typed_self_reference_is_accepted() {
        for code in [
            quote!(
                pub fn run(self: &Self) {}
            ),
            quote!(
                pub fn run(self: &mut Self) {}
            )
        ] {
            assert!(function(code).is_ok());
        }
    }

    #[test]
    fn argument_types_an_entry_point_cannot_read() {
        assert_rejected(
            function(quote!(
                pub fn greet(&self, name: &str) {}
            )),
            "`&str` is not an argument type"
        );
        assert_rejected(
            function(quote!(
                pub fn greet(&self, name: impl Into<String>) {}
            )),
            "`impl Trait` is not an argument type"
        );
        assert_rejected(
            function(quote!(
                pub fn greet(&self, (a, b): (u8, u8)) {}
            )),
            "is not an argument name"
        );
    }

    #[test]
    fn offchain_where_it_does_not_fit() {
        assert_rejected(
            function(quote!(
                #[odra(offchain)]
                pub fn balances() {}
            )),
            "an offchain function reads the contract's state"
        );
        assert_rejected(
            function(quote!(
                #[odra(offchain, payable)]
                pub fn balances(&self) {}
            )),
            "`payable` and `non_reentrant` do not apply to it"
        );
        assert_rejected(
            function(quote!(
                #[odra(offchain, non_reentrant)]
                pub fn balances(&self) {}
            )),
            "`payable` and `non_reentrant` do not apply to it"
        );
        let item = quote!(
            impl Token {
                #[odra(offchain)]
                pub fn balances(&self) {}
            }
        );
        assert_rejected(
            module(quote!(factory = on), item),
            "offchain functions are not supported in a factory module"
        );
        let item = quote!(
            impl IToken for Token {
                #[odra(offchain)]
                fn balances(&self) {}
            }
        );
        assert_rejected(
            module(quote!(), item),
            "a trait impl cannot have offchain functions"
        );
        let item = quote!(
            pub trait Token {
                #[odra(offchain)]
                fn balances(&self);
            }
        );
        assert_rejected(
            module(quote!(), item),
            "an external contract has no offchain functions"
        );
    }

    #[test]
    fn malformed_odra_attributes() {
        assert_rejected(
            function(quote!(
                #[odra]
                pub fn pay(&mut self) {}
            )),
            "malformed odra attribute"
        );
        assert_rejected(
            function(quote!(
                #[odra(payable = true)]
                pub fn pay(&mut self) {}
            )),
            "odra attribute `payable` takes no value"
        );
    }

    #[test]
    fn odra_types_that_cannot_be_serialized() {
        assert_rejected(
            TypeIR::try_from(&quote!(
                struct Point(u32, u32);
            )),
            "an Odra type's fields are named"
        );
        assert_rejected(
            TypeIR::try_from(&quote!(
                enum Code {
                    Big = 70000
                }
            )),
            "a discriminant is an integer literal from 0 to 65535"
        );
        assert_rejected(
            TypeIR::try_from(&quote!(
                struct Wrapper<T> {
                    value: T
                }
            )),
            "an Odra type has no generic parameters"
        );
    }

    #[test]
    fn a_factory_value_other_than_on_or_off() {
        let module = quote!(
            pub struct Token;
        );
        assert_rejected(
            ModuleStructIR::try_from((&quote!(factory = yes), &module)),
            "`yes` is not a `factory` value"
        );
    }

    #[test]
    fn delegate_blocks_that_do_not_forward_a_signature() {
        let impl_with = |delegate: proc_macro2::TokenStream| {
            module(
                quote!(),
                quote!(
                    impl Token {
                        delegate! { #delegate }
                    }
                )
            )
        };
        assert_rejected(
            impl_with(quote!(to self.ownable { pub fn owner(&self) -> Address; })),
            "a delegated function is written without `pub`"
        );
        assert_rejected(
            impl_with(quote!(to self.ownable { fn owner(&self) -> Address { todo!() } })),
            "a delegated function has no body"
        );
        assert_rejected(
            impl_with(quote!(self.ownable { fn owner(&self) -> Address; })),
            "expected `to`; a `delegate!` block is `to self.<field> {"
        );
        assert_rejected(
            impl_with(quote!(to self.ownable.inner { fn owner(&self) -> Address; })),
            "a `delegate!` block forwards to a field of the module"
        );
    }

    #[test]
    fn delegated_arguments_are_forwarded() {
        let ir = module(
            quote!(),
            quote!(
                impl Token {
                    delegate! {
                        to self.ownable {
                            fn transfer_ownership(&mut self, new_owner: Address);
                        }
                    }
                }
            )
        )
        .unwrap();
        let code = ir.self_code().to_string();
        let call = quote!({ self.ownable.transfer_ownership(new_owner) }).to_string();
        assert!(code.contains(&call), "{code}");
    }

    #[test]
    fn impl_blocks_with_something_else_than_entry_points() {
        assert_rejected(
            module(
                quote!(),
                quote!(
                    impl Token {
                        println!("x");
                    }
                )
            ),
            "`println!` is not allowed in a module impl"
        );
        assert_rejected(
            module(
                quote!(name = "Token"),
                quote!(
                    impl Token {}
                )
            ),
            "`name` is not allowed on an impl block"
        );
        assert_rejected(
            module(
                quote!(),
                quote!(
                    #[odra(payable)]
                    impl Token {}
                )
            ),
            "not on the impl block"
        );
    }

    #[test]
    fn external_contracts_with_more_than_signatures() {
        let item = quote!(
            pub trait Token {
                fn total(&self) -> U256;
            }
        );
        assert_rejected(
            ModuleImplIR::try_new_trait(&quote!(name = "X"), syn::parse2(item).unwrap()),
            "`#[odra::external_contract]` takes no arguments"
        );
        assert_rejected(
            module(
                quote!(),
                quote!(
                    pub trait Token: Clone {
                        fn total(&self) -> U256;
                    }
                )
            ),
            "an external contract trait has no supertraits"
        );
    }

    #[test]
    fn module_structs_that_cannot_be_stored() {
        assert_rejected(
            ModuleStructIR::try_from((
                &quote!(),
                &quote!(
                    pub struct Token(Var<u32>);
                )
            )),
            "a module's fields are named"
        );
        assert_rejected(
            ModuleStructIR::try_from((
                &quote!(),
                &quote!(
                    pub struct Token {
                        #[odra(payable)]
                        value: Var<u32>
                    }
                )
            )),
            "not on a field"
        );
    }

    #[test]
    fn an_unknown_flag_is_named_without_spaces() {
        assert_rejected(
            function(quote!(
                #[odra(odra::payable)]
                pub fn pay(&mut self) {}
            )),
            "unknown odra attribute `odra::payable`"
        );
    }

    #[test]
    fn events_that_cannot_be_emitted() {
        use crate::ast::OdraEventItem;
        let event = |code: proc_macro2::TokenStream| OdraEventItem::try_from(&code);
        assert_rejected(
            event(quote!(
                pub struct Transfer(Address);
            )),
            "an event's fields are named"
        );
        assert_rejected(
            event(quote!(
                pub struct Transfer<T> {
                    to: T
                }
            )),
            "an event has no generic parameters"
        );
        assert_rejected(
            event(quote!(
                pub struct Transfer {
                    #[odra(payable)]
                    to: Address
                }
            )),
            "not on a field"
        );
    }

    /// Every error of the input is reported at once, in source order, so one build shows all
    /// the mistakes.
    #[test]
    fn all_errors_are_reported_together_in_order() {
        let messages = |result: syn::Result<ModuleImplIR>| match result {
            Ok(_) => panic!("accepted"),
            Err(error) => error.into_iter().map(|e| e.to_string()).collect::<Vec<_>>()
        };
        let reserved = messages(module(
            quote!(),
            quote!(
                impl Token {
                    pub fn env(&self) {}
                    pub async unsafe fn address(&self) {}
                }
            )
        ));
        assert_eq!(reserved.len(), 4, "{reserved:?}");
        assert!(reserved[0].contains("`env` is a reserved entry point name"));
        assert!(reserved[1].contains("`address` is a reserved entry point name"));
        assert!(reserved[2].contains("`async` is not allowed on `address`"));
        assert!(reserved[3].contains("`unsafe` is not allowed on `address`"));

        let both = messages(module(
            quote!(),
            quote!(
                impl (u32, u32) {
                    pub fn env(&self) {}
                }
            )
        ));
        assert_eq!(both.len(), 2, "{both:?}");
        assert!(both[0].contains("an Odra impl block is for a named module"));
        assert!(both[1].contains("`env` is a reserved entry point name"));
    }
}

/// What is emitted in place of the user's impl block or trait.
mod self_code {
    use crate::ir::ModuleImplIR;
    use quote::{quote, ToTokens};

    fn self_code(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> String {
        let ir = ModuleImplIR::try_from((&attr, &item)).unwrap();
        ir.self_code().into_token_stream().to_string()
    }

    #[test]
    fn delegated_functions_are_written_out_and_odra_attributes_dropped() {
        let actual = self_code(
            quote!(),
            quote!(
                impl Token {
                    #[odra(payable)]
                    pub fn deposit(&mut self) {}
                    delegate! {
                        to self.ownable {
                            fn owner(&self) -> Address;
                        }
                    }
                }
            )
        );
        let expected = quote!(
            impl Token {
                pub fn deposit(&mut self) {}
                #[doc = " Delegated. See `self.ownable.owner()` for details."]
                pub fn owner(&self) -> Address {
                    self.ownable.owner()
                }
            }
        );
        assert_eq!(actual, expected.to_string());
    }

    #[test]
    fn odra_attributes_are_dropped_from_a_trait() {
        let actual = self_code(
            quote!(),
            quote!(
                pub trait Token {
                    #[odra(payable)]
                    fn deposit(&mut self);
                }
            )
        );
        assert_eq!(
            actual,
            quote!(
                pub trait Token {
                    fn deposit(&mut self);
                }
            )
            .to_string()
        );
    }
}

/// The name of a module generated next to the user's one: the module name in snake case, digits
/// kept with the letters before them.
mod generated_mod {
    use super::super::generated_mod_ident;

    fn ident(module: &str) -> String {
        let module = syn::Ident::new(module, proc_macro2::Span::call_site());
        generated_mod_ident(&module, "exec_parts").to_string()
    }

    #[test]
    fn digits_stay_with_the_letters_before_them() {
        assert_eq!(ident("Erc20"), "__erc20_exec_parts");
        assert_eq!(ident("ERC20"), "__erc20_exec_parts");
        assert_eq!(ident("Cep78Token"), "__cep78_token_exec_parts");
        assert_eq!(ident("NFT721Pack"), "__nft721_pack_exec_parts");
    }
}
