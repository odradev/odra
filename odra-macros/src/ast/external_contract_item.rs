use crate::ast::contract_ref_item::RefItem;
use crate::ast::host_ref_item::HostRefItem;
use crate::ast::parts_utils::{UsePreludeItem, UseSuperItem};
use crate::ast::ref_utils;
use crate::ast::test_parts::{PartsModuleItem, TestPartsReexportItem};
use crate::ir::ModuleImplIR;
use derive_try_from_ref::TryFromRef;
use quote::TokenStreamExt;

#[derive(syn_derive::ToTokens)]
pub struct ExternalContractImpl {
    /// The trait itself, so it can be used as a bound or implemented by a module.
    trait_item: proc_macro2::TokenStream,
    ref_item: RefItem,
    ref_trait_impl: TraitImplItem,
    test_parts: TestPartsItem,
    test_parts_reexport: TestPartsReexportItem
}

impl TryFrom<&'_ ModuleImplIR> for ExternalContractImpl {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        let trait_item = ir.self_code()?;
        Ok(Self {
            // Only the generated refs may use the trait; do not warn about that.
            trait_item: quote::quote!(#[allow(dead_code)] #trait_item),
            ref_item: ir.try_into()?,
            ref_trait_impl: TraitImplItem::contract_ref(ir)?,
            test_parts: ir.try_into()?,
            test_parts_reexport: ir.try_into()?
        })
    }
}

/// `impl Trait for XxxRef { .. }` - makes the ref usable wherever the trait is required.
struct TraitImplItem {
    trait_ident: syn::Ident,
    ref_ident: syn::Ident,
    functions: Vec<syn::ItemFn>
}

impl TraitImplItem {
    fn contract_ref(ir: &ModuleImplIR) -> syn::Result<Self> {
        Ok(Self {
            trait_ident: ir.module_ident()?,
            ref_ident: ir.contract_ref_ident()?,
            functions: ir
                .functions()?
                .iter()
                .map(|f| ref_utils::contract_function_item(f, true))
                .collect()
        })
    }

    fn host_ref(ir: &ModuleImplIR) -> syn::Result<Self> {
        Ok(Self {
            trait_ident: ir.module_ident()?,
            ref_ident: ir.host_ref_ident()?,
            functions: ir
                .host_functions()?
                .iter()
                .map(|f| ref_utils::host_function_item(f, true))
                .collect()
        })
    }
}

impl quote::ToTokens for TraitImplItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let trait_ident = &self.trait_ident;
        let ref_ident = &self.ref_ident;
        let functions = &self.functions;
        tokens.append_all(quote::quote! {
            impl #trait_ident for #ref_ident {
                #(#functions)*
            }
        })
    }
}

impl TryFrom<&'_ ModuleImplIR> for HostTraitImplItem {
    type Error = syn::Error;

    fn try_from(ir: &'_ ModuleImplIR) -> Result<Self, Self::Error> {
        Ok(Self(TraitImplItem::host_ref(ir)?))
    }
}

#[derive(syn_derive::ToTokens)]
struct HostTraitImplItem(TraitImplItem);

#[derive(syn_derive::ToTokens, TryFromRef)]
#[source(ModuleImplIR)]
#[err(syn::Error)]
struct TestPartsItem {
    parts_module: PartsModuleItem,
    #[syn(braced)]
    #[default]
    brace_token: syn::token::Brace,
    #[syn(in = brace_token)]
    #[default]
    use_super: UseSuperItem,
    #[syn(in = brace_token)]
    #[default]
    use_prelude: UsePreludeItem,
    #[syn(in = brace_token)]
    host_ref: HostRefItem,
    #[syn(in = brace_token)]
    host_trait_impl: HostTraitImplItem
}

#[cfg(test)]
mod test {
    use crate::test_utils;

    use super::ExternalContractImpl;

    #[test]
    fn external_contract_impl() {
        let ir = test_utils::mock::ext_contract();
        let item = ExternalContractImpl::try_from(&ir).unwrap();
        let expected = quote::quote! {
            #[allow(dead_code)]
            pub trait Token {
                fn balance_of(&self, owner: Address) -> U256;
            }

            /// [Token] Contract Ref.
            pub struct TokenContractRef {
                env: odra::prelude::Rc<odra::ContractEnv>,
                address: odra::prelude::Address,
                attached_value: odra::casper_types::U512,
            }

            impl odra::ContractRef for TokenContractRef {
                fn new(env: odra::prelude::Rc<odra::ContractEnv>, address: odra::prelude::Address) -> Self {
                    Self {
                        env,
                        address,
                        attached_value: odra::casper_types::U512::zero()
                    }
                }

                fn address(&self) -> &odra::prelude::Address {
                    &self.address
                }

                fn with_tokens(&self, tokens: odra::casper_types::U512) -> Self {
                    Self {
                        address: self.address,
                        env: self.env.clone(),
                        attached_value: tokens,
                    }
                }
            }

            impl TokenContractRef {
                pub fn balance_of(&self, owner: Address) -> U256 {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("balance_of"),
                            false,
                            {
                                let mut named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = named_args.insert("amount", self.attached_value);
                                }
                                odra::args::EntrypointArgument::insert_runtime_arg(owner, "owner", &mut named_args);
                                named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
                }
            }

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaErrors for TokenContractRef {}

            #[automatically_derived]
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::schema::SchemaEvents for TokenContractRef {}

            impl Token for TokenContractRef {
                fn balance_of(&self, owner: Address) -> U256 {
                    self.env.call_contract(
                        self.address,
                        odra::CallDef::new(
                            odra::prelude::string::String::from("balance_of"),
                            false,
                            {
                                let mut named_args = odra::casper_types::RuntimeArgs::new();
                                if self.attached_value > odra::casper_types::U512::zero() {
                                    let _ = named_args.insert("amount", self.attached_value);
                                }
                                odra::args::EntrypointArgument::insert_runtime_arg(owner, "owner", &mut named_args);
                                named_args
                            }
                        )
                        .with_amount(self.attached_value),
                    )
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            mod __token_test_parts {
                use super::*;
                use odra::prelude::*;

                /// [Token] Host Ref.
                pub struct TokenHostRef {
                    address: odra::prelude::Address,
                    env: odra::host::HostEnv,
                    attached_value: odra::casper_types::U512
                }

                impl odra::host::HostRef for TokenHostRef {
                    fn new(address: odra::prelude::Address, env: odra::host::HostEnv) -> Self {
                        Self {
                            address,
                            env,
                            attached_value: Default::default()
                        }
                    }

                    fn with_tokens(&self, tokens: odra::casper_types::U512) -> Self {
                        Self {
                            address: self.address,
                            env: self.env.clone(),
                            attached_value: tokens
                        }
                    }

                    fn contract_address(&self) -> odra::prelude::Address {
                        self.address
                    }

                    fn env(&self) -> &odra::host::HostEnv {
                        &self.env
                    }

                    fn get_event<T>(&self, index: i32) -> Result<T, odra::EventError>
                    where
                        T: odra::casper_types::bytesrepr::FromBytes + odra::casper_event_standard::EventInstance,
                    {
                        self.env.get_event(self, index)
                    }

                    fn last_call(&self) -> odra::ContractCallResult {
                        self.env.last_call_result(self.address)
                    }
                }

                impl TokenHostRef {
                    pub fn balance_of(&self, owner: Address) -> U256 {
                        self.try_balance_of(owner).unwrap()
                    }
                }

                impl TokenHostRef {
                    /// Does not fail in case of error, returns `odra::OdraResult` instead.
                    pub fn try_balance_of(&self, owner: Address) -> OdraResult<U256> {
                        self.env.call_contract(
                            self.address,
                            odra::CallDef::new(
                                odra::prelude::string::String::from("balance_of"),
                                false,
                                {
                                    let mut named_args = odra::casper_types::RuntimeArgs::new();
                                    if self.attached_value > odra::casper_types::U512::zero() {
                                        let _ = named_args.insert("amount", self.attached_value);
                                    }
                                    odra::args::EntrypointArgument::insert_runtime_arg(owner, "owner", &mut named_args);
                                    named_args
                                }
                            ).with_amount(self.attached_value),
                        )
                    }
                }

                impl Token for TokenHostRef {
                    fn balance_of(&self, owner: Address) -> U256 {
                        self.try_balance_of(owner).unwrap()
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub use __token_test_parts::*;
        };

        test_utils::assert_eq(item, expected);
    }
}
