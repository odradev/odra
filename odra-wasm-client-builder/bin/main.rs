use std::{fs::File, io::Write, process::Command};

use odra_schema::casper_contract_schema::{Argument, ContractSchema, NamedCLType, Type};
use quote::{format_ident, ToTokens};
use syn::parse_quote;

fn main() -> Result<(), String> {
    // Read the path to a schema file
    let schema_path = std::env::args().nth(1).expect("No schema path provided");
    let schema = std::fs::read_to_string(schema_path).expect("Failed to read schema file");

    let contract_schema: ContractSchema =
        serde_json::from_str(&schema).expect("Failed to parse schema");

    let client_name = format_ident!("{}WasmClient", contract_schema.contract_name);

    let struct_definition = quote::quote! {
        #[wasm_bindgen]
        pub struct #client_name {
            wasm_client: odra_wasm_client::OdraWasmClient,
            wallet: odra_wasm_client::CasperWallet,
            address: odra_wasm_client::types::Address
        }
    };

    let implementation = contract_schema.entry_points.iter().map(|ep| {
        let ret_ty = named_cl_type_to_wasm_type(&ep.return_ty);
        let name = format_ident!("{}", &ep.name);
        let name_str = &ep.name;
        let ident = format_ident!("{}", ep.name);
        let args = ep.arguments.iter().map(parse_arg).collect::<Vec<_>>();
        let rt_args = ep.arguments
            .iter()
            .map(|arg| {
                let arg_name = format_ident!("{}", arg.name);
                if is_primitive(&arg.ty) {
                    parse_quote!(#arg_name)
                } else {
                    parse_quote!(*#arg_name)
                }
            })
            .collect::<Vec<syn::Expr>>();
        let rt_names = ep.arguments.iter().map(|arg| format_ident!("{}", arg.name)).collect::<Vec<_>>();

        let returns_value = ep.return_ty.0 != NamedCLType::Unit;
        let deser_ty = named_cl_type_to_type(&ep.return_ty);

        if returns_value {
            quote::quote! {
                #[wasm_bindgen]
                pub async fn #name(&self, #(#args),*) -> Result<#ret_ty, odra_wasm_client::wasm_bindgen::JsError> {
                    if !self.wallet.request_connection().await.is_ok() {
                        return Err(odra_wasm_client::wasm_bindgen::JsError::new("Could not connect to the wallet"));
                    }
                    let cl_value = self
                        .wasm_client
                        .call_entry_point_with_proxy(&self.wallet, *self.address, #name_str, casper_types::runtime_args! {
                            #(stringify!(#rt_names) => #rt_args),*
                        })
                        .await?;

                    let result = <#deser_ty as casper_types::bytesrepr::FromBytes>::from_bytes(&cl_value.inner_bytes()[4..])
                        .map_err(|err| odra_wasm_client::wasm_bindgen::JsError::new(&format!("{:?}", err)))?;
                    Ok(result.0.into())
                }
            }
        } else {
            quote::quote! {
                #[wasm_bindgen]
                pub async fn #ident(&self, #(#args),*) -> Result<odra_wasm_client::types::TransactionHash, odra_wasm_client::wasm_bindgen::JsError> {
                    if !self.wallet.request_connection().await.is_ok() {
                        return Err(odra_wasm_client::wasm_bindgen::JsError::new("Could not connect to the wallet"));
                    }
                    self.wasm_client
                        .call_entry_point(
                            &self.wallet,
                            *self.address,
                            #name_str,
                            casper_types::runtime_args! {
                                #(stringify!(#rt_names) => #rt_args),*
                            }
                        )
                        .await
                }
            }
        }

    }).collect::<Vec<_>>();

    let code = quote::quote! {
        use odra_wasm_client::wasm_bindgen as wasm_bindgen;
        use odra_wasm_client::wasm_bindgen_futures as wasm_bindgen_futures;
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
    };
    
    // write to token stream
    let mut file = File::create("../wasm_client/src/lib.rs").map_err(|e| e.to_string())?;
    file.write_all(code.to_token_stream().to_string().as_bytes())
        .map_err(|e| e.to_string())?;

    Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg("pkg-web")
        .arg("../wasm_client")
        .status()
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn parse_arg(fn_arg: &Argument) -> syn::FnArg {
    let arg_ident = format_ident!("{}", fn_arg.name);
    let ty = named_cl_type_to_wasm_type(&fn_arg.ty);
    parse_quote!(#arg_ident: #ty)
}

fn named_cl_type_to_wasm_type(ty: &Type) -> syn::Path {
    match &ty.0 {
        NamedCLType::Bool => parse_quote!(bool),
        NamedCLType::I32 => parse_quote!(i32),
        NamedCLType::I64 => parse_quote!(i64),
        NamedCLType::U8 => parse_quote!(u8),
        NamedCLType::U32 => parse_quote!(u32),
        NamedCLType::U64 => parse_quote!(u64),
        NamedCLType::U128 => parse_quote!(odra_wasm_client::types::U128),
        NamedCLType::U256 => parse_quote!(odra_wasm_client::types::U256),
        NamedCLType::U512 => parse_quote!(odra_wasm_client::types::U512),
        NamedCLType::Unit => parse_quote!(odra_wasm_client::types::Unit),
        NamedCLType::String => parse_quote!(String),
        NamedCLType::Key => parse_quote!(odra_wasm_client::types::Address),
        NamedCLType::URef => todo!(),
        NamedCLType::PublicKey => todo!(),
        NamedCLType::Option(_named_cltype) => todo!(),
        NamedCLType::List(_named_cltype) => todo!(),
        NamedCLType::ByteArray(_) => todo!(),
        NamedCLType::Result { ok: _, err: _ } => todo!(),
        NamedCLType::Map { key: _, value: _ } => todo!(),
        NamedCLType::Tuple1(_) => todo!(),
        NamedCLType::Tuple2(_) => todo!(),
        NamedCLType::Tuple3(_) => todo!(),
        NamedCLType::Custom(_) => todo!()
    }
}

fn named_cl_type_to_type(ty: &Type) -> syn::Path {
    match &ty.0 {
        NamedCLType::Bool => parse_quote!(bool),
        NamedCLType::I32 => parse_quote!(i32),
        NamedCLType::I64 => parse_quote!(i64),
        NamedCLType::U8 => parse_quote!(u8),
        NamedCLType::U32 => parse_quote!(u32),
        NamedCLType::U64 => parse_quote!(u64),
        NamedCLType::U128 => parse_quote!(casper_types::U128),
        NamedCLType::U256 => parse_quote!(casper_types::U256),
        NamedCLType::U512 => parse_quote!(casper_types::U512),
        NamedCLType::Unit => parse_quote!(casper_types::Unit),
        NamedCLType::String => parse_quote!(String),
        NamedCLType::Key => parse_quote!(casper_types::Key),
        NamedCLType::URef => todo!(),
        NamedCLType::PublicKey => todo!(),
        NamedCLType::Option(_named_cltype) => todo!(),
        NamedCLType::List(_named_cltype) => todo!(),
        NamedCLType::ByteArray(_) => todo!(),
        NamedCLType::Result { ok: _, err: _ } => todo!(),
        NamedCLType::Map { key: _, value: _ } => todo!(),
        NamedCLType::Tuple1(_) => todo!(),
        NamedCLType::Tuple2(_) => todo!(),
        NamedCLType::Tuple3(_) => todo!(),
        NamedCLType::Custom(_) => todo!()
    }
}


fn is_primitive(ty: &Type) -> bool {
    match &ty.0 {
        NamedCLType::Bool => true,
        NamedCLType::I32 => true,
        NamedCLType::I64 => true,
        NamedCLType::U8 => true,
        NamedCLType::U32 => true,
        NamedCLType::U64 => true,
        NamedCLType::String => true,
       _ => false
    }
}