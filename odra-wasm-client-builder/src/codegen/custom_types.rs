use convert_case::{Case, Casing};
use odra_schema::casper_contract_schema::{CustomType, EnumVariant, StructMember, UserError};
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{parse_quote, punctuated::Punctuated, Token};

use crate::types::{OdraType, WasmType};

pub fn user_errors(contract_ident: &str, errors: &[UserError]) -> TokenStream {
    let errors = errors
        .iter()
        .map(|err| {
            let name = format_ident!("{}", err.name);
            let code = err.discriminant as isize;
            let description = err.description.clone().unwrap_or_default();
            quote::quote! {
                #[doc = #description]
                #name = #code
            }
        })
        .collect::<Vec<_>>();

    let name = format_ident!("{}Errors", contract_ident);
    quote::quote! {
        #[wasm_bindgen]
        #[derive(Debug, Clone)]
        pub enum #name {
            #(#errors),*
        }
    }
}

pub fn types_def<T: IntoIterator<Item = CustomType>>(types: T) -> Vec<TokenStream> {
    types
        .into_iter()
        .map(|custom_type| match custom_type {
            CustomType::Struct {
                name,
                members,
                description
            } => struct_def(&name.0, &members, description.unwrap_or_default()),
            CustomType::Enum {
                name,
                variants,
                description
            } => enum_def(&name.0, &variants, description.unwrap_or_default())
        })
        .collect::<Vec<_>>()
}

fn struct_def(name: &str, members: &[StructMember], description: String) -> TokenStream {
    let type_name = format_ident!("{}", name);
    let struct_fields = members.iter().map(field_def).collect::<Vec<syn::Field>>();

    let setters_getters = members
        .iter()
        .filter_map(|field| {
            let odra_ty = OdraType::from(&field.ty);
            if odra_ty.is_copyable() {
                return None;
            }

            let setter_code = setter_code(field);
            let getter_code = getter_code(field);
            Some(quote::quote! {
                #setter_code
                #getter_code
            })
        })
        .collect::<Vec<TokenStream>>();

    let field_args = members.iter().map(field_arg).collect::<Vec<syn::Field>>();

    let fields_init = members.iter().map(field_init).collect::<Vec<TokenStream>>();

    let field_names = members
        .iter()
        .map(|field| format_ident!("{}", field.name))
        .collect::<Vec<syn::Ident>>();

    let fields_deser = field_names
        .iter()
        .map(|ident| {
            parse_quote!(let (#ident, bytes) = odra_wasm_client::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;)
        })
        .collect::<Vec<syn::Stmt>>();

    let fields_ser = field_names
        .iter()
        .map(|ident| {
            parse_quote!(result.extend(odra_wasm_client::casper_types::bytesrepr::ToBytes::to_bytes(&self.#ident)?);)
        })
        .collect::<Vec<syn::Stmt>>();

    let fields_len = field_names
        .iter()
        .map(|ident| parse_quote!(result += self.#ident.serialized_length();))
        .collect::<Vec<syn::Stmt>>();

    quote::quote! {
        #[doc = #description]
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[wasm_bindgen(getter_with_clone)]
        pub struct #type_name {
            #(#struct_fields),*
        }

        #[wasm_bindgen]
        impl #type_name {
            #[wasm_bindgen(constructor)]
            pub fn new(#(#field_args),*) -> Result<Self, JsError> {
                Ok(Self {
                    #(#fields_init),*
                })
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

        impl odra_wasm_client::types::IntoOdraValue<#type_name> for #type_name {
            fn into_odra_value(self) -> Result<#type_name, JsError> {
                Ok(self)
            }
        }

        impl odra_wasm_client::types::IntoWasmValue<#type_name> for #type_name {
            fn to_wasm_value(self) -> Self {
                self
            }
        }
    }
}

fn enum_def(name: &str, variants: &[EnumVariant], description: String) -> TokenStream {
    let type_name = format_ident!("{}", name);
    let variants_expr = variants
        .iter()
        .map(|v| {
            let ident = format_ident!("{}", v.name);
            let discriminant = v.discriminant as isize;
            parse_quote!(#ident = #discriminant)
        })
        .collect::<Vec<syn::Expr>>();
    let match_arms = variants
        .iter()
        .enumerate()
        .map(|(v_idx, v)| {
            let v_idx: u8 = v_idx as u8;
            let ident = format_ident!("{}", v.name);
            quote::quote!(#v_idx => Ok((Self::#ident, bytes)))
        })
        .collect::<Punctuated<TokenStream, Token![,]>>();

    quote::quote! {
        #[doc = #description]
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[wasm_bindgen]
        pub enum #type_name {
            #(#variants_expr),*
        }

        impl odra_wasm_client::casper_types::bytesrepr::FromBytes for #type_name {
            fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), odra_wasm_client::casper_types::bytesrepr::Error> {
                let (result, bytes): (u8, _) = odra_wasm_client::casper_types::bytesrepr::FromBytes::from_bytes(bytes)?;
                match result {
                    #match_arms,
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

        impl odra_wasm_client::types::IntoOdraValue<#type_name> for #type_name {
            fn into_odra_value(self) -> Result<#type_name, JsError> {
                Ok(self)
            }
        }

        impl odra_wasm_client::types::IntoWasmValue<#type_name> for #type_name {
            fn to_wasm_value(self) -> Self {
                self
            }
        }
    }
}

fn field_def(member: &StructMember) -> syn::Field {
    let odra_ty = OdraType::from(&member.ty);
    let field_name = format_ident!("{}", member.name);
    let js_name = member.name.to_case(Case::Camel);
    if odra_ty.is_copyable() {
        parse_quote!(#[wasm_bindgen(js_name = #js_name)] pub #field_name: #odra_ty)
    } else {
        parse_quote!(#[wasm_bindgen(js_name = #js_name)] #field_name: #odra_ty)
    }
}

fn field_init(member: &StructMember) -> proc_macro2::TokenStream {
    let field_name = format_ident!("{}", member.name);
    let odra_ty = OdraType::from(&member.ty);
    let wasm_ty = WasmType::from(&member.ty);
    if wasm_ty == odra_ty {
        return parse_quote!(#field_name);
    }
    quote::quote! {
        #field_name: odra_wasm_client::types::IntoOdraValue::into_odra_value(#field_name)?
    }
}

fn field_arg(member: &StructMember) -> syn::Field {
    let ty = WasmType::from(&member.ty);
    let field_name = format_ident!("{}", member.name);
    let js_name = member.name.to_case(Case::Camel);
    parse_quote!(#[wasm_bindgen(js_name = #js_name)] #field_name: #ty)
}

fn getter_code(member: &StructMember) -> proc_macro2::TokenStream {
    let ty = WasmType::from(&member.ty);
    let ident = format_ident!("{}", member.name);

    quote::quote! {
        #[wasm_bindgen(getter)]
        pub fn #ident(&self) -> #ty {
            odra_wasm_client::types::IntoWasmValue::to_wasm_value(self.#ident.clone())
        }
    }
}

fn setter_code(member: &StructMember) -> proc_macro2::TokenStream {
    let ty = WasmType::from(&member.ty);
    let ident = format_ident!("set_{}", member.name);
    let field_name = format_ident!("{}", member.name);

    quote::quote! {
        #[wasm_bindgen(setter)]
        pub fn #ident(&mut self, value: #ty) {
            self.#field_name = odra_wasm_client::types::IntoOdraValue::into_odra_value(value).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::codegen::custom_types::field_def;
    use odra_schema::casper_contract_schema::{NamedCLType, Type};

    #[test]
    fn test_field_def() {
        let field = StructMember {
            name: "test".to_string(),
            description: None,
            ty: Type(NamedCLType::U128)
        };
        let tokens = field_def(&field);
        let expected = parse_quote!(#[wasm_bindgen(js_name = "test")] test: odra_wasm_client::casper_types::U128);
        pretty_assertions::assert_eq!(tokens, expected);

        let field = StructMember {
            name: "test_field_rust_style".to_string(),
            description: None,
            ty: Type(NamedCLType::String)
        };
        let tokens = field_def(&field);
        let expected = parse_quote!(#[wasm_bindgen(js_name = "testFieldRustStyle")] pub test_field_rust_style: String);
        pretty_assertions::assert_eq!(tokens, expected);
    }
}
