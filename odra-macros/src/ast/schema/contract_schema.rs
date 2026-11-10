use crate::ir::ModuleStructIR;
use quote::ToTokens;

/// Emits `mod __xxx_contract_schema { fn casper_contract_schema() }`, compiled on the host only
/// when the crate is built for this module: the symbol `cargo odra schema` calls to write the
/// contract's schema file, with the name and version of `#[odra::module(..)]` or of the crate.
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
        let qualified_module_name = format!(
            "{}::{}",
            crate::utils::attr::current_crate_name(),
            module_name
        );

        let item = quote::quote! {
            #[automatically_derived]
            #[cfg(all(
                not(target_arch = "wasm32"),
                any(odra_module = #module_name, odra_module = #qualified_module_name)
            ))]
            mod #mod_ident {
                use super::*;

                #[no_mangle]
                fn casper_contract_schema() -> odra::schema::casper_contract_schema::ContractSchema {
                    let version = match #version {
                        "" => env!("CARGO_PKG_VERSION"),
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

        Ok(Self {
            mod_ident: module.contract_schema_mod_ident(),
            name: module
                .contract_name()
                .unwrap_or_else(|| module_ident.to_string()),
            module_ident,
            version: module.contract_version().unwrap_or_default()
        })
    }
}

#[cfg(test)]
mod tests;
