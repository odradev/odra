use crate::types::{
    named_cl_type_to_odra_type, named_cl_type_to_type, named_cl_type_to_wasm_type, value_type,
    ValueType
};
use convert_case::{Case, Casing};
use odra_schema::casper_contract_schema::{Argument, ContractSchema, Entrypoint, NamedCLType};
use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};
use syn::parse_quote;

pub fn client(contract_schema: &ContractSchema) -> proc_macro2::TokenStream {
    let client_name = format_ident!("{}WasmClient", contract_schema.contract_name);
    let struct_definition = client_struct_def(&client_name);
    let implementation = contract_schema
        .entry_points
        .iter()
        .map(entry_point_def)
        .collect::<Vec<_>>();

    quote::quote! {
        use odra_wasm_client::wasm_bindgen as wasm_bindgen;
        use odra_wasm_client::wasm_bindgen_futures as wasm_bindgen_futures;
        use odra_wasm_client::JsValueSerdeExt;
        use wasm_bindgen::prelude::*;
        #struct_definition

        #[wasm_bindgen]
        impl #client_name {
            #[wasm_bindgen(constructor)]
            pub fn new(wasm_client: odra_wasm_client::OdraWasmClient, address: odra_wasm_client::types::Address) -> Self {
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

fn client_struct_def<T: ToTokens>(client_name: &T) -> proc_macro2::TokenStream {
    quote::quote! {
        #[wasm_bindgen]
        pub struct #client_name {
            wasm_client: odra_wasm_client::OdraWasmClient,
            wallet: odra_wasm_client::CasperWallet,
            address: odra_wasm_client::types::Address
        }
    }
}

fn entry_point_def(ep: &Entrypoint) -> proc_macro2::TokenStream {
    let entry_point_ident = format_ident!("{}", &ep.name);
    let entry_point_str = &ep.name;
    let js_name = ep.name.to_case(Case::Camel);
    let args = ep
        .arguments
        .iter()
        .map(parse_entry_point_arg)
        .collect::<Vec<_>>();
    let rt_args = ep
        .arguments
        .iter()
        .map(runtime_arg)
        .collect::<Vec<TokenStream>>();
    let parse_js_input = ep
        .arguments
        .iter()
        .map(parse_js_value_arg)
        .collect::<Vec<Option<syn::Stmt>>>();
    let ret_ty = named_cl_type_to_wasm_type(&ep.return_ty);
    let deser_ty = named_cl_type_to_type(&ep.return_ty);
    let ret_expr = return_expr(ep);

    let returns_value = ep.return_ty.0 != NamedCLType::Unit;
    if returns_value {
        quote::quote! {
            #[wasm_bindgen(js_name = #js_name)]
            pub async fn #entry_point_ident(&self, #(#args),*) -> Result<#ret_ty, odra_wasm_client::wasm_bindgen::JsError> {
                if !self.wallet.request_connection().await.is_ok() {
                    return Err(odra_wasm_client::wasm_bindgen::JsError::new("Could not connect to the wallet"));
                }
                #(#parse_js_input)*
                let cl_value = self
                    .wasm_client
                    .call_entry_point_with_proxy(&self.wallet, *self.address, #entry_point_str, casper_types::runtime_args! {
                        #(#rt_args),*
                    })
                    .await?;

                let result = <#deser_ty as casper_types::bytesrepr::FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
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
                        casper_types::runtime_args! {
                            #(#rt_args),*
                        }
                    )
                    .await
            }
        }
    }
}

fn parse_entry_point_arg(fn_arg: &Argument) -> syn::FnArg {
    let arg_ident = format_ident!("{}", fn_arg.name);
    let ty = named_cl_type_to_wasm_type(&fn_arg.ty);
    parse_quote!(#arg_ident: #ty)
}

fn parse_js_value_arg(arg: &Argument) -> Option<syn::Stmt> {
    let arg_name = format_ident!("{}", arg.name);
    let odra_type = named_cl_type_to_odra_type(&arg.ty);
    match value_type(&arg.ty) {
        ValueType::JsValue | ValueType::JsValueList => {
            if matches!(arg.ty.0, NamedCLType::List(_))
                || matches!(arg.ty.0, NamedCLType::ByteArray(_))
            {
                Some(parse_quote! {
                    let #arg_name: #odra_type = #arg_name
                        .into_iter()
                        .map(|js_value| {
                            js_value
                                .into_serde()
                                .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))
                        })
                        .collect::<Result<_, _>>()?;
                })
            } else {
                Some(
                    parse_quote!(#arg_name.into_serde::<#odra_type>().map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))?)
                )
            }
        }
        _ => None
    }
}

fn runtime_arg(arg: &Argument) -> TokenStream {
    let arg_name = format_ident!("{}", arg.name);
    match value_type(&arg.ty) {
        ValueType::Serializable => parse_quote!(stringify!(#arg_name) => *#arg_name),
        _ => parse_quote!(stringify!(#arg_name) => #arg_name)
    }
}

fn return_expr(ep: &Entrypoint) -> syn::Expr {
    match value_type(&ep.return_ty) {
        ValueType::Primitive => parse_quote!(Ok(result.0.into())),
        ValueType::JsValueList => parse_quote! {
            result.0.into_iter()
                .map(|v| JsValue::from_serde(&v))
                .collect::<Result<Vec<JsValue>, _>>()
                .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))
        },
        ValueType::JsValue => parse_quote! {
            JsValue::from_serde(&result.0).map_err(|err| JsError::new(&format!("{:?}", err)))
        },
        ValueType::Serializable => parse_quote!(Ok(result.0.into())),
        ValueType::Bytes => parse_quote!(Ok(result.0.to_vec())),
        ValueType::Custom => parse_quote!(Ok(result.0))
    }
}
