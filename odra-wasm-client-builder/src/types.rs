use odra_schema::casper_contract_schema::{NamedCLType, Type};
use quote::{format_ident, ToTokens};
use syn::parse_quote;

pub fn named_cl_type_to_wasm_type(ty: &Type) -> syn::Path {
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
        NamedCLType::URef => parse_quote!(odra_wasm_client::types::URef),
        NamedCLType::PublicKey => parse_quote!(odra_wasm_client::types::PublicKey),
        NamedCLType::Option(named_cltype) => {
            let inner = named_cl_type_to_wasm_type(&Type(*named_cltype.clone()));
            parse_quote!(Option<#inner>)
        }
        NamedCLType::List(named_cltype) => {
            let t = *named_cltype.to_owned();
            match t {
                NamedCLType::Option(_named_cltype) => parse_quote!(Vec<JsValue>),
                NamedCLType::List(_named_cltype) => parse_quote!(Vec<JsValue>),
                NamedCLType::ByteArray(_) => parse_quote!(Vec<JsValue>),
                NamedCLType::Result { ok: _, err: _ } => parse_quote!(Vec<JsValue>),
                NamedCLType::Map { key: _, value: _ } => parse_quote!(Vec<JsValue>),
                NamedCLType::Tuple1(_) => parse_quote!(Vec<JsValue>),
                NamedCLType::Tuple2(_) => parse_quote!(Vec<JsValue>),
                NamedCLType::Tuple3(_) => parse_quote!(Vec<JsValue>),
                NamedCLType::Custom(_) => parse_quote!(Vec<JsValue>),
                _ => {
                    let inner = named_cl_type_to_wasm_type(&Type(*named_cltype.clone()));
                    parse_quote!(Vec<#inner>)
                }
            }
        }
        NamedCLType::ByteArray(_n) => parse_quote!(Vec<u8>),
        NamedCLType::Result { ok, err } => {
            let ok = named_cl_type_to_wasm_type(&Type(*ok.clone()));
            let err = named_cl_type_to_wasm_type(&Type(*err.clone()));
            parse_quote!(Result<#ok, #err>)
        }
        NamedCLType::Map { key: _, value: _ } => parse_quote!(JsValue),
        NamedCLType::Tuple1(_) => parse_quote!(JsValue),
        NamedCLType::Tuple2(_) => parse_quote!(JsValue),
        NamedCLType::Tuple3(_) => parse_quote!(JsValue),
        NamedCLType::Custom(name) => {
            let ident = format_ident!("{}", name);
            parse_quote!(#ident)
        }
    }
}

pub fn named_cl_type_to_odra_type(ty: &Type) -> syn::Type {
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
        NamedCLType::Unit => parse_quote!(()),
        NamedCLType::String => parse_quote!(String),
        NamedCLType::Key => parse_quote!(odra_core::prelude::Address),
        NamedCLType::URef => parse_quote!(casper_types::URef),
        NamedCLType::PublicKey => parse_quote!(casper_types::PublicKey),
        NamedCLType::Option(named_cltype) => {
            let inner = named_cl_type_to_odra_type(&Type(*named_cltype.clone()));
            parse_quote!(Option<#inner>)
        }
        NamedCLType::List(named_cltype) => {
            let inner = named_cl_type_to_odra_type(&Type(*named_cltype.clone()));
            parse_quote!(Vec<#inner>)
        }
        NamedCLType::ByteArray(_n) => parse_quote!(Vec<u8>),
        NamedCLType::Result { ok, err } => {
            let ok = named_cl_type_to_odra_type(&Type(*ok.clone()));
            let err = named_cl_type_to_odra_type(&Type(*err.clone()));
            parse_quote!(Result<#ok, #err>)
        }
        NamedCLType::Map { key: _, value: _ } => parse_quote!(JsValue),
        NamedCLType::Tuple1(ty) => {
            let inner = named_cl_type_to_odra_type(&Type(*ty[0].clone()));
            parse_quote!( (#inner,) )
        }
        NamedCLType::Tuple2(ty) => {
            let inner = named_cl_type_to_odra_type(&Type(*ty[0].clone()));
            let inner2 = named_cl_type_to_odra_type(&Type(*ty[1].clone()));
            println!(
                "Tuple2 types detected: {:?}, {:?}",
                inner.to_token_stream().to_string(),
                inner2.to_token_stream().to_string()
            );
            parse_quote!( (#inner, #inner2) )
        }
        NamedCLType::Tuple3(ty) => {
            let inner = named_cl_type_to_odra_type(&Type(*ty[0].clone()));
            let inner2 = named_cl_type_to_odra_type(&Type(*ty[1].clone()));
            let inner3 = named_cl_type_to_odra_type(&Type(*ty[2].clone()));
            parse_quote!( (#inner, #inner2, #inner3) )
        }
        NamedCLType::Custom(name) => {
            let ident = format_ident!("{}", name);
            parse_quote!(#ident)
        }
    }
}

pub fn named_cl_type_to_type(ty: &Type) -> syn::Type {
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
        NamedCLType::URef => parse_quote!(casper_types::URef),
        NamedCLType::PublicKey => parse_quote!(casper_types::PublicKey),
        NamedCLType::Option(named_cltype) => {
            let inner = named_cl_type_to_type(&Type(*named_cltype.clone()));
            println!("Option type detected: {:?}", inner);
            parse_quote!(Option<#inner>)
        }
        NamedCLType::List(named_cltype) => {
            let inner = named_cl_type_to_type(&Type(*named_cltype.clone()));
            parse_quote!(Vec<#inner>)
        }
        NamedCLType::ByteArray(n) => {
            let n: usize = n.to_owned().try_into().unwrap_or_default();
            parse_quote!([u8; #n])
        }
        NamedCLType::Result { ok, err } => {
            let ok = named_cl_type_to_type(&Type(*ok.clone()));
            let err = named_cl_type_to_type(&Type(*err.clone()));
            parse_quote!(Result<#ok, #err>)
        }
        NamedCLType::Map { key, value } => {
            let key = named_cl_type_to_type(&Type(*key.clone()));
            let value = named_cl_type_to_type(&Type(*value.clone()));
            parse_quote!(std::collections::BTreeMap<#key, #value>)
        }
        NamedCLType::Tuple1(ty) => {
            let inner = named_cl_type_to_type(&Type(*ty[0].clone()));
            parse_quote!( (#inner,) )
        }
        NamedCLType::Tuple2(ty) => {
            let inner = named_cl_type_to_type(&Type(*ty[0].clone()));
            let inner2 = named_cl_type_to_type(&Type(*ty[1].clone()));
            parse_quote!( (#inner, #inner2) )
        }
        NamedCLType::Tuple3(ty) => {
            let inner = named_cl_type_to_type(&Type(*ty[0].clone()));
            let inner2 = named_cl_type_to_type(&Type(*ty[1].clone()));
            let inner3 = named_cl_type_to_type(&Type(*ty[2].clone()));
            parse_quote!( (#inner, #inner2, #inner3) )
        }
        NamedCLType::Custom(name) => {
            let ident = format_ident!("{}", name);
            parse_quote!(#ident)
        }
    }
}

pub enum ValueType {
    Primitive,
    JsValue,
    JsValueList,
    Serializable,
    Bytes,
    Custom
}

pub fn value_type(ty: &Type) -> ValueType {
    match &ty.0 {
        NamedCLType::Bool => ValueType::Primitive,
        NamedCLType::I32 => ValueType::Primitive,
        NamedCLType::I64 => ValueType::Primitive,
        NamedCLType::U8 => ValueType::Primitive,
        NamedCLType::U32 => ValueType::Primitive,
        NamedCLType::U64 => ValueType::Primitive,
        NamedCLType::String => ValueType::Primitive,
        NamedCLType::U128 => ValueType::Serializable,
        NamedCLType::U256 => ValueType::Serializable,
        NamedCLType::U512 => ValueType::Serializable,
        NamedCLType::Unit => ValueType::Serializable,
        NamedCLType::Key => ValueType::Serializable,
        NamedCLType::URef => ValueType::Serializable,
        NamedCLType::PublicKey => ValueType::Serializable,
        NamedCLType::Option(named_cltype) => value_type(&Type(*named_cltype.clone())),
        NamedCLType::List(named_cltype) => match &**named_cltype {
            NamedCLType::Option(_named_cltype) => ValueType::JsValueList,
            NamedCLType::List(_named_cltype) => ValueType::JsValueList,
            NamedCLType::ByteArray(_) => ValueType::JsValueList,
            NamedCLType::Result { ok: _, err: _ } => ValueType::JsValueList,
            NamedCLType::Map { key: _, value: _ } => ValueType::JsValueList,
            NamedCLType::Tuple1(_) => ValueType::JsValueList,
            NamedCLType::Tuple2(_) => ValueType::JsValueList,
            NamedCLType::Tuple3(_) => ValueType::JsValueList,
            NamedCLType::Custom(_) => ValueType::JsValueList,
            _ => value_type(&Type(*named_cltype.clone()))
        },
        NamedCLType::ByteArray(_n) => ValueType::Bytes,
        NamedCLType::Result { ok: _, err: _ } => ValueType::Serializable,
        NamedCLType::Custom(_name) => ValueType::Custom,
        _ => ValueType::JsValue
    }
}
