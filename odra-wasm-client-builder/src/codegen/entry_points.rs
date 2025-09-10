use crate::types::{OdraType, WasmType};
use convert_case::{Case, Casing};
use odra_schema::casper_contract_schema::{ContractSchema, Entrypoint, NamedCLType};
use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};

pub fn client(contract_schema: &ContractSchema) -> TokenStream {
    let client_name = format_ident!("{}WasmClient", contract_schema.contract_name);
    let struct_definition = client_struct_def(&client_name);
    let implementation = contract_schema
        .entry_points
        .iter()
        .map(entry_point_def)
        .collect::<Vec<_>>();

    quote::quote! {
        #struct_definition

        #[wasm_bindgen]
        impl #client_name {
            #[wasm_bindgen(constructor)]
            pub fn new(#[wasm_bindgen(js_name = "wasmClient")] wasm_client: odra_wasm_client::OdraWasmClient, address: odra_wasm_client::types::Address) -> Self {
                #client_name {
                    wasm_client,
                    wallet: odra_wasm_client::CasperWallet::default(),
                    address
                }
            }

            #(#implementation)*
        }
    }
}

fn client_struct_def<T: ToTokens>(client_name: &T) -> TokenStream {
    quote::quote! {
        #[wasm_bindgen]
        pub struct #client_name {
            wasm_client: odra_wasm_client::OdraWasmClient,
            wallet: odra_wasm_client::CasperWallet,
            address: odra_wasm_client::types::Address
        }
    }
}

fn entry_point_def(ep: &Entrypoint) -> TokenStream {
    let entry_point_ident = format_ident!("{}", &ep.name);
    let entry_point_str = &ep.name;
    let js_name = ep.name.to_case(Case::Camel);
    let args = ep
        .arguments
        .iter()
        .map(WasmType::parse_entry_point_arg)
        .collect::<Vec<_>>();
    let rt_args = ep
        .arguments
        .iter()
        .map(WasmType::runtime_arg)
        .collect::<Vec<TokenStream>>();
    let parse_js_input = ep
        .arguments
        .iter()
        .map(WasmType::parse_js_value_arg)
        .collect::<Vec<Option<syn::Stmt>>>();
    let ret_ty = WasmType::from(&ep.return_ty);
    let deser_ty = OdraType::from(&ep.return_ty);
    let ret_expr = WasmType::parse_return_expr(&ep.return_ty);

    let returns_value = ep.return_ty.0 != NamedCLType::Unit;
    let is_mut = ep.is_mutable;

    if is_mut && returns_value {
        quote::quote! {
            panic!("Mutable entry points with return values are not supported");
        }
    } else if returns_value {
        quote::quote! {
            #[wasm_bindgen(js_name = #js_name)]
            pub async fn #entry_point_ident(&self, #(#args),*) -> Result<#ret_ty, odra_wasm_client::wasm_bindgen::JsError> {
                #(#parse_js_input)*
                let cl_value = self
                    .wasm_client
                    .call_entry_point_with_proxy(*self.address, #entry_point_str, odra_wasm_client::casper_types::runtime_args! {
                        #(#rt_args),*
                    })
                    .await?;

                let result = <#deser_ty as odra_wasm_client::casper_types::bytesrepr::FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
                    .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))?;
                #ret_expr
            }
        }
    } else {
        quote::quote! {
            #[wasm_bindgen(js_name = #js_name)]
            pub async fn #entry_point_ident(&self, #(#args),*) -> Result<odra_wasm_client::types::TransactionHash, odra_wasm_client::wasm_bindgen::JsError> {
                if !self.wallet.request_connection().await.is_ok() {
                    return Err(odra_wasm_client::wasm_bindgen::JsError::new("Could not connect to the wallet"));
                }
                #(#parse_js_input)*
                self.wasm_client
                    .call_entry_point(
                        &self.wallet,
                        *self.address,
                        #entry_point_str,
                        odra_wasm_client::casper_types::runtime_args! {
                            #(#rt_args),*
                        }
                    )
                    .await
            }
        }
    }
}
