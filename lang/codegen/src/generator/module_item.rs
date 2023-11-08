use derive_more::From;
use quote::{quote, quote_spanned};

use crate::{generator::module_item::node::NodeItem, poet::OdraPoetUsingStruct, GenerateCode};

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

        let node = self.generate_code_using::<NodeItem>();

        quote! {
            #instance
            #item_struct

            #node

            impl odra::types::OdraItem for #struct_ident {
                fn is_module() -> bool {
                    true
                }
                #[cfg(not(target_arch = "wasm32"))]
                fn events() -> odra::prelude::vec::Vec<odra::types::contract_def::Event> {
                    <Self as odra::types::contract_def::HasEvents>::events()
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            impl odra::types::contract_def::HasEvents for #struct_ident {
                fn events() -> odra::prelude::vec::Vec<odra::types::contract_def::Event> {
                    let mut events = odra::prelude::collections::BTreeSet::new();
                    #(
                        events.insert(<#module_events as odra::types::event::OdraEvent>::schema());
                    )*
                    #(
                        events.extend(<#submodules_events as odra::types::OdraItem>::events());
                    )*
                    #(
                        events.extend(<#mappings_events as odra::types::OdraItem>::events());
                    )*
                    events.iter().map(Clone::clone).collect::<odra::prelude::vec::Vec<_>>()
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
                pub mapping2: Mapping<u32, odra::prelude::string::String>,
                pub mapping3: Mapping<u32, odra::types::U256>,
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
                pub mapping2: Mapping<u32, odra::prelude::string::String>,
                pub mapping3: Mapping<u32, odra::types::U256>,
                pub submodule: Submodule
            }

            #[cfg(not (target_arch = "wasm32"))]
            impl odra::types::contract_def::Node for Module {
                const IS_LEAF: bool = false;

                const COUNT: u32 =
                    <Variable<u32> as odra::types::contract_def::Node>::COUNT +
                    <Mapping<u32, Mapping<u32, MappedModule> > as odra::types::contract_def::Node>::COUNT +
                    <Mapping<u32, odra::prelude::string::String> as odra::types::contract_def::Node>::COUNT +
                    <Mapping<u32, odra::types::U256> as odra::types::contract_def::Node>::COUNT +
                    <Submodule as odra::types::contract_def::Node>::COUNT;


                fn __keys() -> odra::prelude::vec::Vec<odra::prelude::string::String> {
                    let mut result = odra::prelude::vec![];
                    if <Variable<u32> as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(odra::prelude::string::String::from("variable"));
                    } else {
                        result.extend(<Variable<u32> as odra::types::contract_def::Node>::__keys().iter().map(|k| odra::utils::create_key("variable" , k)))
                    }
                    if <Mapping<u32, Mapping<u32, MappedModule> > as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(odra::prelude::string::String::from("mapping"));
                    } else {
                        result.extend(<Mapping<u32, Mapping<u32, MappedModule> > as odra::types::contract_def::Node>::__keys().iter().map(|k| odra::utils::create_key("mapping" , k)))
                    }
                    if <Mapping<u32, odra::prelude::string::String> as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(odra::prelude::string::String::from("mapping2"));
                    } else {
                        result.extend(<Mapping<u32, odra::prelude::string::String> as odra::types::contract_def::Node>::__keys().iter().map(|k| odra::utils::create_key("mapping2" , k)))
                    }
                    if <Mapping<u32, odra::types::U256> as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(odra::prelude::string::String::from("mapping3"));
                    } else {
                        result.extend(<Mapping<u32, odra::types::U256> as odra::types::contract_def::Node>::__keys().iter().map(|k| odra::utils::create_key("mapping3" , k)))
                    }
                    if <Submodule as odra::types::contract_def::Node>::IS_LEAF {
                        result.push(odra::prelude::string::String::from("submodule"));
                    } else {
                        result.extend(<Submodule as odra::types::contract_def::Node>::__keys().iter().map(|k| odra::utils::create_key("submodule" , k)))
                    }
                    result
                }
            }

            impl odra::types::OdraItem for Module {
                fn is_module() -> bool {
                    true
                }

                #[cfg(not (target_arch = "wasm32"))]
                fn events () -> odra::prelude::vec::Vec<odra::types::contract_def::Event> {
                    <Self as odra::types::contract_def::HasEvents>::events()
                }
            }

            #[cfg(not (target_arch = "wasm32"))]
            impl odra::types::contract_def::HasEvents for Module {
                fn events() -> odra::prelude::vec::Vec<odra::types::contract_def::Event> {
                    let mut events = odra::prelude::collections::BTreeSet::new();
                    events.insert(<A as odra::types::event::OdraEvent>::schema());
                    events.insert(<B as odra::types::event::OdraEvent>::schema());
                    events.insert(<C as odra::types::event::OdraEvent>::schema());
                    events.extend(<Submodule as odra::types::OdraItem>::events());
                    events.extend(<MappedModule as odra::types::OdraItem>::events());
                    events.extend(<odra::prelude::string::String as odra::types::OdraItem>::events());
                    events.extend(<odra::types::U256 as odra::types::OdraItem>::events());
                    events.iter().map(Clone::clone).collect::<odra::prelude::vec::Vec<_>>()
                }
            }
        };
        let actual = super::ModuleStruct::from(&module_struct).generate_code();
        pretty_assertions::assert_eq!(actual.to_string(), expected.to_string());
    }
}
