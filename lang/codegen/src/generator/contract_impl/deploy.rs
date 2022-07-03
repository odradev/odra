use derive_more::From;
use odra_ir::contract_item::contract_impl::ContractImpl;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::GenerateCode;

#[derive(From)]
pub struct Deploy<'a> {
    contract: &'a ContractImpl,
}

as_ref_for_contract_impl_generator!(Deploy);

impl GenerateCode for Deploy<'_> {
    fn generate_code(&self) -> TokenStream {
        let struct_ident = self.contract.ident();
        let struct_name = struct_ident.to_string();
        let struct_name_lowercased = struct_name.to_lowercase();
        let ref_ident = format_ident!("{}Ref", struct_ident);

        let register_entrypoints = self.contract
            .methods()
            .iter()
            .filter_map(|item| match item {
                odra_ir::contract_item::impl_item::ImplItem::Constructor(_) => None,
                odra_ir::contract_item::impl_item::ImplItem::Method(method) => Some(method),
                odra_ir::contract_item::impl_item::ImplItem::Other(_) => None,
            })
            .map(|entrypoint| {
                let ident = &entrypoint.ident;
                let name = quote!(stringify!(#ident).to_string());
                let return_value = match &entrypoint.ret {
                    syn::ReturnType::Default => quote!(None),
                    syn::ReturnType::Type(_, _) => quote! {
                        let bytes = odra::types::bytesrepr::ToBytes::to_bytes(&result).unwrap();
                        Some(odra::types::bytesrepr::Bytes::from(bytes))
                    }
                };
                let args = &entrypoint.args.iter().map(|arg| {
                    let pat = &*arg.pat;
                    quote!(args.get(stringify!(#pat)).cloned().unwrap().into_t().unwrap(),)
                }).flatten().collect::<TokenStream>();

                quote! {
                    entrypoints.insert(#name, |name, args| {
                        let instance = <#struct_ident as odra::instance::Instance>::instance(name.as_str());
                        let result = instance.#ident(#args);
                        #return_value
                    });
                }
            })
            .flatten()
            .collect::<TokenStream>();

        quote! {
            #[cfg(all(test, feature = "wasm-test"))]
            impl #struct_ident {
                fn deploy(name: &str, args: odra::types::RuntimeArgs) -> #ref_ident {
                    let container = odra::ContractContainer {
                        name: #struct_name_lowercased.to_string(),
                        wasm_path: format!("{}.wasm", #struct_name_lowercased),
                        args,
                    };

                    let address = odra::TestEnv::register_contract(&container);
                    #ref_ident { address }
                }
            }

            #[cfg(all(test, feature = "mock-vm"))]
            impl #struct_ident {
                fn deploy(name: &str, args: odra::types::RuntimeArgs) -> #ref_ident {
                    use std::collections::HashMap;
                    use odra::types::{bytesrepr::Bytes, RuntimeArgs};

                    pub type EntrypointCall = fn(String, RuntimeArgs) -> Option<Bytes>;

                    let mut entrypoints: HashMap<String, EntrypointCall> = HashMap::new();

                    #register_entrypoints

                    let address = odra::TestEnv::register_contract(name, entrypoints, args);
                    #ref_ident { address }
                }
            }
        }
    }
}
