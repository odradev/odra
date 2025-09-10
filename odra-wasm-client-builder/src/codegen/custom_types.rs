use odra_schema::casper_contract_schema::{CustomType, EnumVariant, StructMember};
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::parse_quote;

use crate::types::{OdraType, WasmType};

pub fn types_def<T: IntoIterator<Item = CustomType>>(types: T) -> Vec<TokenStream> {
    types
        .into_iter()
        .map(|custom_type| match custom_type {
            CustomType::Struct { name, members, .. } => struct_def(&name.0, &members),
            CustomType::Enum { name, variants, .. } => enum_def(&name.0, &variants)
        })
        .collect::<Vec<_>>()
}

fn struct_def(name: &str, members: &[StructMember]) -> TokenStream {
    let type_name = format_ident!("{}", name);
    let struct_fields = members
        .iter()
        .map(|field| {
            let ty = OdraType::from(&field.ty);
            ty.field(field)
        })
        .collect::<Vec<syn::Field>>();

    let setters_getters = members
        .iter()
        .filter_map(|field| {
            let odra_ty = OdraType::from(&field.ty);

            if odra_ty.is_cloneable() {
                return None;
            }

            let setter_code = WasmType::setter_code(field);
            let getter_code = WasmType::getter_code(field);
            Some(quote::quote! {
                #setter_code
                #getter_code
            })
        })
        .collect::<Vec<TokenStream>>();

    let fields = members
        .iter()
        .map(WasmType::field_def)
        .collect::<Vec<syn::Field>>();

    let fields_init = members
        .iter()
        .map(WasmType::field_init)
        .collect::<Vec<TokenStream>>();

    let field_names = members
        .iter()
        .map(|field| {
            let field_name = format_ident!("{}", field.name);
            parse_quote!(#field_name)
        })
        .collect::<Vec<syn::Expr>>();

    let fields_deser = members
        .iter()
        .map(|field| {
            let field_name = format_ident!("{}", field.name);
            parse_quote!(let (#field_name, bytes) = odra_wasm_client::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;)
        })
        .collect::<Vec<syn::Stmt>>();

    let fields_ser = members
        .iter()
        .map(|field| {
            let field_name = format_ident!("{}", field.name);
            parse_quote!(result.extend(odra_wasm_client::casper_types::bytesrepr::ToBytes::to_bytes(&self.#field_name)?);)
        })
        .collect::<Vec<syn::Stmt>>();

    let fields_len = members
        .iter()
        .map(|field| {
            let field_name = format_ident!("{}", field.name);
            parse_quote!(result += self.#field_name.serialized_length();)
        })
        .collect::<Vec<syn::Stmt>>();

    quote::quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[wasm_bindgen(getter_with_clone)]
        pub struct #type_name {
            #(#struct_fields),*
        }

        #[wasm_bindgen]
        impl #type_name {
            #[wasm_bindgen(constructor)]
            pub fn new(#(#fields),*) -> Self {
                Self {
                    #(#fields_init),*
                }
            }

            #[wasm_bindgen(js_name = "toJson")]
            pub fn to_json(&self) -> JsValue {
                JsValue::from_serde(self).unwrap_or(JsValue::null())
            }

            #(#setters_getters)*
        }

        impl odra_wasm_client::casper_types::bytesrepr::FromBytes for #type_name {
            fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra_wasm_client::casper_types::bytesrepr::Error> {
                #(#fields_deser)*
                Ok((Self { #(#field_names),* }, bytes))
            }
        }

        impl odra_wasm_client::casper_types::bytesrepr::ToBytes for #type_name {
            fn to_bytes(
                &self
            ) -> Result<Vec<u8>, odra_wasm_client::casper_types::bytesrepr::Error> {
                let mut result = Vec::with_capacity(self.serialized_length());
                #(#fields_ser)*
                Ok(result)
            }
            fn serialized_length(&self) -> usize {
                let mut result = 0;
                #(#fields_len)*
                result
            }
        }

        impl odra_wasm_client::casper_types::CLTyped for #type_name {
            fn cl_type() -> odra_wasm_client::casper_types::CLType {
                odra_wasm_client::casper_types::CLType::Any
            }
        }
    }
}

fn enum_def(name: &str, variants: &[EnumVariant]) -> TokenStream {
    let type_name = format_ident!("{}", name);
    let variants_expr = variants
        .iter()
        .map(|v| {
            let ident = format_ident!("{}", v.name);
            let discriminant = v.discriminant;
            parse_quote!(#ident = #discriminant)
        })
        .collect::<Vec<syn::Expr>>();
    let match_arms = variants
        .iter()
        .map(|v| {
            let ident = format_ident!("{}", v.name);
            parse_quote!(x if x == Self::#ident as u8 => Ok((Self::#ident, bytes)))
        })
        .collect::<Vec<syn::Expr>>();

    quote::quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[wasm_bindgen]
        pub enum #type_name {
            #(#variants_expr),*
        }

        #[wasm_bindgen]
        impl #type_name {
            #[wasm_bindgen(js_name = "toJson")]
            pub fn to_json(&self) -> JsValue {
                JsValue::from_serde(self).unwrap_or(JsValue::null())
            }
        }

        impl odra_wasm_client::casper_types::bytesrepr::FromBytes for #type_name {
            fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra_wasm_client::casper_types::bytesrepr::Error> {
                let (result, bytes): (u8, _) = odra_wasm_client::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                match result {
                    #(#match_arms),*
                    _ => Err(odra_wasm_client::casper_types::bytesrepr::Error::Formatting)
                }
            }
        }

        impl odra_wasm_client::casper_types::bytesrepr::ToBytes for #type_name {
            fn to_bytes(
                &self
            ) -> Result<Vec<u8>, odra_wasm_client::casper_types::bytesrepr::Error> {
                Ok(vec![(self.clone() as u8)])
            }
            fn serialized_length(&self) -> usize {
                odra_wasm_client::casper_types::bytesrepr::U8_SERIALIZED_LENGTH
            }
        }

        impl odra_wasm_client::casper_types::CLTyped for #type_name {
            fn cl_type() -> odra_wasm_client::casper_types::CLType {
                odra_wasm_client::casper_types::CLType::U8
            }
        }
    }
}
