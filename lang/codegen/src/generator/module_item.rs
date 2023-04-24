use derive_more::From;
use quote::{quote, quote_spanned};

use crate::{generator::module_item::composer::ModuleComposer, GenerateCode};

mod composer;

#[derive(From)]
pub struct ModuleStruct<'a> {
    pub contract: &'a odra_ir::module::ModuleStruct
}

impl GenerateCode for ModuleStruct<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let events = &self.contract.events;
        let module_events = events.events.iter().collect::<Vec<_>>();
        let submodules_events = events.submodules_events.iter().collect::<Vec<_>>();
        let mappings_events = events.mappings_events.iter().collect::<Vec<_>>();

        let item_struct = &self.contract.item;

        let struct_ident = &item_struct.ident;
        let span = item_struct.ident.span();
        let instance = if self.contract.is_instantiable && !self.contract.skip_instance {
            quote_spanned!(span => #[derive(odra::Instance, Clone)])
        } else {
            quote!(#[derive(Clone)])
        };

        let composer =
            <ModuleComposer as GenerateCode>::generate_code(&ModuleComposer::from(self.contract));

        quote! {
            #instance
            #item_struct

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
            #composer
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
        let config = ModuleConfiguration {
            events,
            ..Default::default()
        };

        let item_struct = syn::parse2::<syn::ItemStruct>(input).unwrap();
        let module_struct = odra_ir::module::ModuleStruct::from(item_struct);
        let module_struct = module_struct.with_config(config).unwrap();

        let expected = quote::quote! {
            #[derive(odra::Instance)]
            pub struct Module {
                pub variable: Variable<u32>,
                pub mapping: Mapping<u32, Mapping<u32, MappedModule> >,
                pub mapping2: Mapping<u32, String>,
                pub submodule: Submodule
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
        };
        let actual = super::ModuleStruct::from(&module_struct).generate_code();
        pretty_assertions::assert_eq!(actual.to_string(), expected.to_string());
    }
}
