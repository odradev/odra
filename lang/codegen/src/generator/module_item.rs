use derive_more::From;
use quote::{quote, quote_spanned};

use crate::{generator::module_item::node::Node, poet::OdraPoetUsingStruct, GenerateCode};

mod node;

#[derive(From)]
pub struct ModuleStruct<'a> {
    pub module: &'a odra_ir::module::ModuleStruct
}

as_ref_for_contract_struct_generator!(ModuleStruct);

impl GenerateCode for ModuleStruct<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let events = &self.module.events;
        let module_events = events.events.iter().collect::<Vec<_>>();
        let submodules_events = events.submodules_events.iter().collect::<Vec<_>>();
        let mappings_events = events.mappings_events.iter().collect::<Vec<_>>();

        let item_struct = &self.module.item;

        let struct_ident = &item_struct.ident;
        let span = item_struct.ident.span();
        let instance = if self.module.is_instantiable {
            quote_spanned!(span => #[derive(odra::Instance, Clone)])
        } else {
            quote!(#[derive(Clone)])
        };

        let node = self.generate_code_using::<Node>();

        quote! {
            #instance
            #item_struct

            #node

            impl odra::OdraItem for #struct_ident {
                fn is_module() -> bool {
                    true
                }
                #[cfg(feature = "casper")]
                fn events() -> Vec<odra::types::contract_def::Event> {
                    <Self as odra::types::contract_def::HasEvents>::events()
                }
            }
            #[cfg(feature = "casper")]
            impl odra::types::contract_def::HasEvents for #struct_ident {
                fn events() -> Vec<odra::types::contract_def::Event> {
                    let mut events = vec![];
                    #(
                        events.push(<#module_events as odra::types::event::OdraEvent>::schema());
                    )*
                    #(
                        events.extend(<#submodules_events as odra::OdraItem>::events());
                    )*
                    #(
                        events.extend(<#mappings_events as odra::OdraItem>::events());
                    )*
                    events.dedup();
                    events
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use odra_ir::module::{ModuleConfiguration, ModuleEvents};

    use crate::generator::GenerateCode;

    #[test]
    fn test() {
        let input = quote::quote! {
            pub struct Module {
                pub variable: Variable<u32>,
                pub mapping: Mapping<u32, Mapping<u32, MappedModule>>,
                pub mapping2: Mapping<u32, String>,
                pub submodule: Submodule
            }
        };
        let events_input = quote::quote!(events = [A, B, C]);
        let events = syn::parse2::<ModuleEvents>(events_input).unwrap();
        let config = ModuleConfiguration { events };

        let item_struct = syn::parse2::<syn::ItemStruct>(input).unwrap();
        let module_struct = odra_ir::module::ModuleStruct::try_from(item_struct).unwrap();
        let module_struct = module_struct.with_config(config).unwrap();

        let expected = quote::quote! {
            #[derive(odra::Instance, Clone)]
            pub struct Module {
                pub variable: Variable<u32>,
                pub mapping: Mapping<u32, Mapping<u32, MappedModule> >,
                pub mapping2: Mapping<u32, String>,
                pub submodule: Submodule
            }

            impl odra::types::contract_def::Node for Module {
                const IS_LEAF: bool = false;

                const COUNT: u32 =
                    <Variable<u32> as odra::types::contract_def::Node>::COUNT +
                    <Mapping<u32, Mapping<u32, MappedModule> > as odra::types::contract_def::Node>::COUNT +
                    <Mapping<u32, String> as odra::types::contract_def::Node>::COUNT +
                    <Submodule as odra::types::contract_def::Node>::COUNT;


                fn __keys() -> Vec<String> {
                    let mut result = vec![];
                    if <Variable<u32> as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(String::from("variable"));
                    } else {
                        result.extend(<Variable<u32> as odra::types::contract_def::Node>::__keys().iter().map(|k| format!("{}#{}", "variable", k)))
                    }
                    if <Mapping<u32, Mapping<u32, MappedModule> > as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(String::from("mapping"));
                    } else {
                        result.extend(<Mapping<u32, Mapping<u32, MappedModule> > as odra::types::contract_def::Node>::__keys().iter().map(|k| format!("{}#{}", "mapping", k)))
                    }
                    if <Mapping<u32, String> as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(String::from("mapping2"));
                    } else {
                        result.extend(<Mapping<u32, String> as odra::types::contract_def::Node>::__keys().iter().map(|k| format!("{}#{}", "mapping2", k)))
                    }
                    if <Submodule as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(String::from("submodule"));
                    } else {
                        result.extend(<Submodule as odra::types::contract_def::Node>::__keys().iter().map(|k| format!("{}#{}", "submodule", k)))
                    }
                    result
                }
            }

            impl odra::OdraItem for Module {
                fn is_module() -> bool {
                    true
                }

                #[cfg (feature = "casper")]
                fn events () -> Vec<odra::types::contract_def::Event> {
                    <Self as odra::types::contract_def::HasEvents>::events()
                }
            }

            #[cfg (feature = "casper")]
            impl odra::types::contract_def::HasEvents for Module {
                fn events() -> Vec<odra::types::contract_def::Event> {
                    let mut events = vec![];
                    events.push(<A as odra::types::event::OdraEvent>::schema());
                    events.push(<B as odra::types::event::OdraEvent>::schema());
                    events.push(<C as odra::types::event::OdraEvent>::schema());
                    events.extend(<Submodule as odra::OdraItem>::events());
                    events.extend(<MappedModule as odra::OdraItem>::events());
                    events.extend(<String as odra::OdraItem>::events());
                    events.dedup();
                    events
                }
            }

            # [doc = "Composer for the [Module] module."]
            pub struct ModuleComposer {
                namespace: String,
                variable: core::option::Option<Variable<u32> >,
                mapping: core::option::Option<Mapping<u32, Mapping<u32, MappedModule> > >,
                mapping2: core::option::Option<Mapping<u32, String> >,
                submodule: core::option::Option<Submodule>
            }

            impl ModuleComposer {
                pub fn new(namespace: &str, name: &str) -> Self {
                    Self {
                        namespace: format!("{}_{}", name, namespace),
                        variable: core::option::Option::None,
                        mapping: core::option::Option::None,
                        mapping2: core::option::Option::None,
                        submodule: core::option::Option::None
                    }
                }

                pub fn with_variable(mut self, variable: &Variable<u32>) -> Self {
                    self.variable = core::option::Option::Some(variable.clone());
                    self
                }

                pub fn with_mapping(mut self, mapping: &Mapping<u32, Mapping<u32, MappedModule> >) -> Self {
                    self.mapping = core::option::Option::Some(mapping.clone());
                    self
                }

                pub fn with_mapping2(mut self, mapping2: &Mapping<u32, String>) -> Self {
                    self.mapping2 = core::option::Option::Some(mapping2.clone());
                    self
                }

                pub fn with_submodule(mut self, submodule: &Submodule) -> Self {
                    self.submodule = core::option::Option::Some(submodule.clone());
                    self
                }

                pub fn compose(self) -> Module {
                    Module {
                        variable: self.variable.unwrap_or_else(|| odra::Instance::instance(&format!("{}_{}", &self.namespace, stringify!(variable)))),
                        mapping: self.mapping.unwrap_or_else(|| odra::Instance::instance(&format!("{}_{}", &self.namespace, stringify!(mapping)))),
                        mapping2: self.mapping2.unwrap_or_else(|| odra::Instance::instance(&format!("{}_{}", &self.namespace, stringify!(mapping2)))),
                        submodule: self.submodule.unwrap_or_else(|| odra::Instance::instance(&format!("{}_{}", &self.namespace, stringify!(submodule))))
                    }
                }
            }
        };
        let actual = super::ModuleStruct::from(&module_struct).generate_code();
        pretty_assertions::assert_eq!(actual.to_string(), expected.to_string());
    }
}
