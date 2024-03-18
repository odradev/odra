use crate::ir::ModuleStructIR;
use quote::ToTokens;

pub struct SchemaItem {
    mod_ident: syn::Ident,
    module_ident: syn::Ident,
    name: String,
    version: String
}

impl ToTokens for SchemaItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let version = &self.version;
        let ident = &self.module_ident;
        let module_name = &self.module_ident.to_string();
        let mod_ident = &self.mod_ident;

        let item = quote::quote! {
            #[cfg(all(not(target_arch = "wasm32"), odra_module = #module_name))]
            mod #mod_ident {
                use super::*;

                #[no_mangle]
                fn casper_contract_schema() -> odra::schema::casper_contract_schema::ContractSchema {
                    let version = match #version {
                        "" =>  env!("CARGO_PKG_VERSION"),
                        _ => #version
                    };

                    let authors = env!("CARGO_PKG_AUTHORS").to_string()
                        .split(":")
                        .filter_map(|s| if s.is_empty() { None } else { Some(s.trim().to_owned()) })
                        .collect();
                    let repository = env!("CARGO_PKG_REPOSITORY");
                    let homepage = env!("CARGO_PKG_HOMEPAGE");
                    odra::schema::schema::<#ident>(
                        #module_name,
                        #name,
                        version,
                        authors,
                        repository,
                        homepage
                    )
                }
            }
        };

        item.to_tokens(tokens);
    }
}

impl TryFrom<&'_ ModuleStructIR> for SchemaItem {
    type Error = syn::Error;

    fn try_from(module: &'_ ModuleStructIR) -> Result<Self, Self::Error> {
        let module_ident = module.module_ident();

        let name = match module.contract_name().as_str() {
            "" => module_ident.to_string(),
            name => name.to_string()
        };

        Ok(Self {
            mod_ident: module.contract_schema_mod_ident(),
            module_ident,
            name,
            version: module.contract_version()
        })
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils;
    use quote::quote;

    use super::SchemaItem;

    #[test]
    fn contract_schema_item() {
        let module = test_utils::mock::module_definition();
        let item = SchemaItem::try_from(&module).unwrap();
        let expected = quote!(
            #[cfg(all(not(target_arch = "wasm32"), odra_module = "CounterPack"))]
            mod __counter_pack_contract_schema {
                use super::*;

                #[no_mangle]
                fn casper_contract_schema() -> odra::schema::casper_contract_schema::ContractSchema
                {
                    let version = match "0.1.0" {
                        "" => env!("CARGO_PKG_VERSION"),
                        _ => "0.1.0"
                    };
                    let authors = env!("CARGO_PKG_AUTHORS")
                        .to_string()
                        .split(":")
                        .filter_map(|s| if s.is_empty() { None } else { Some(s.trim().to_owned()) })
                        .collect();
                    let repository = env!("CARGO_PKG_REPOSITORY");
                    let homepage = env!("CARGO_PKG_HOMEPAGE");
                    odra::schema::schema::<CounterPack>(
                        "CounterPack",
                        "MyCounterPack",
                        version,
                        authors,
                        repository,
                        homepage
                    )
                }
            }
        );
        test_utils::assert_eq(item, &expected);
    }
}
