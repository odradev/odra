use odra_schema::casper_contract_schema::{NamedCLType, Type};
use quote::{format_ident, ToTokens};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WasmType {
    Bool,
    I32,
    I64,
    U8,
    U32,
    U64,
    U128,
    U256,
    U512,
    Unit,
    String,
    URef,
    PublicKey,
    JsValue,
    JsValueList,
    Bytes,
    Address,
    Custom(String),
    Option(Box<WasmType>),
    List(Box<WasmType>)
}

impl WasmType {}

impl ToTokens for WasmType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            WasmType::Bool => quote::quote!(bool),
            WasmType::I32 => quote::quote!(i32),
            WasmType::I64 => quote::quote!(i64),
            WasmType::U8 => quote::quote!(u8),
            WasmType::U32 => quote::quote!(u32),
            WasmType::U64 => quote::quote!(u64),
            WasmType::U128 => quote::quote!(odra_wasm_client::types::U128),
            WasmType::U256 => quote::quote!(odra_wasm_client::types::U256),
            WasmType::U512 => quote::quote!(odra_wasm_client::types::U512),
            WasmType::Address => quote::quote!(odra_wasm_client::types::Address),
            WasmType::URef => quote::quote!(odra_wasm_client::types::URef),
            WasmType::PublicKey => quote::quote!(odra_wasm_client::types::PublicKey),
            WasmType::Unit => quote::quote!(()),
            WasmType::String => quote::quote!(String),
            WasmType::JsValue => quote::quote!(JsValue),
            WasmType::JsValueList => quote::quote!(Vec<JsValue>),
            WasmType::Bytes => quote::quote!(Vec<u8>),
            WasmType::Option(inner) => quote::quote!(Option<#inner>),
            WasmType::Custom(name) => {
                let ident = format_ident!("{}", name);
                quote::quote!(#ident)
            }
            WasmType::List(inner) => quote::quote!(Vec<#inner>)
        });
    }
}

impl From<&Type> for WasmType {
    fn from(ty: &Type) -> Self {
        named_cl_type_to_wasm_type(ty)
    }
}

fn named_cl_type_to_wasm_type(ty: &Type) -> WasmType {
    match &ty.0 {
        NamedCLType::Bool => WasmType::Bool,
        NamedCLType::I32 => WasmType::I32,
        NamedCLType::I64 => WasmType::I64,
        NamedCLType::U8 => WasmType::U8,
        NamedCLType::U32 => WasmType::U32,
        NamedCLType::U64 => WasmType::U64,
        NamedCLType::U128 => WasmType::U128,
        NamedCLType::U256 => WasmType::U256,
        NamedCLType::U512 => WasmType::U512,
        NamedCLType::Unit => WasmType::Unit,
        NamedCLType::String => WasmType::String,
        NamedCLType::Key => WasmType::Address,
        NamedCLType::URef => WasmType::URef,
        NamedCLType::PublicKey => WasmType::PublicKey,
        NamedCLType::Option(named_cltype) => {
            let inner = named_cl_type_to_wasm_type(&Type(*named_cltype.clone()));
            WasmType::Option(Box::new(inner))
        }
        NamedCLType::List(named_cltype) => match *named_cltype.to_owned() {
            // List of complex types are not supported so we treat them as JsValueList
            NamedCLType::Option(_)
            | NamedCLType::List(_)
            | NamedCLType::ByteArray(_)
            | NamedCLType::Result { ok: _, err: _ }
            | NamedCLType::Map { key: _, value: _ }
            | NamedCLType::Tuple1(_)
            | NamedCLType::Tuple2(_)
            | NamedCLType::Tuple3(_) => WasmType::JsValueList,
            _ => {
                let inner = named_cl_type_to_wasm_type(&Type(*named_cltype.clone()));
                WasmType::List(Box::new(inner))
            }
        },
        NamedCLType::ByteArray(_) => WasmType::Bytes,
        NamedCLType::Result { ok: _, err: _ } => WasmType::JsValue,
        NamedCLType::Map { key: _, value: _ }
        | NamedCLType::Tuple1(_)
        | NamedCLType::Tuple2(_)
        | NamedCLType::Tuple3(_) => WasmType::JsValue,
        NamedCLType::Custom(name) => WasmType::Custom(name.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_wasm_type_conversion() {
        let wasm_type = WasmType::from(&Type(NamedCLType::I32));
        let expected = WasmType::I32;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::I64));
        let expected = WasmType::I64;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U8));
        let expected = WasmType::U8;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U32));
        let expected = WasmType::U32;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U64));
        let expected = WasmType::U64;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U128));
        let expected = WasmType::U128;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U256));
        let expected = WasmType::U256;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::U512));
        let expected = WasmType::U512;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::Unit));
        let expected = WasmType::Unit;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::String));
        let expected = WasmType::String;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::URef));
        let expected = WasmType::URef;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::PublicKey));
        let expected = WasmType::PublicKey;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::Key));
        let expected = WasmType::Address;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::ByteArray(10)));
        let expected = WasmType::Bytes;
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::Custom("MyType".to_string())));
        let expected = WasmType::Custom("MyType".to_string());
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::Option(Box::new(NamedCLType::U128))));
        let expected = WasmType::Option(Box::new(WasmType::U128));
        assert_eq!(wasm_type, expected);

        let wasm_type = WasmType::from(&Type(NamedCLType::Option(Box::new(NamedCLType::Option(
            Box::new(NamedCLType::U128)
        )))));
        let expected = WasmType::Option(Box::new(WasmType::Option(Box::new(WasmType::U128))));
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::Tuple1([Box::new(NamedCLType::U128)]));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValue;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::Tuple2([
            Box::new(NamedCLType::U128),
            Box::new(NamedCLType::U256)
        ]));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValue;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::Tuple3([
            Box::new(NamedCLType::U128),
            Box::new(NamedCLType::U256),
            Box::new(NamedCLType::U512)
        ]));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValue;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::Map {
            key: Box::new(NamedCLType::U128),
            value: Box::new(NamedCLType::String)
        });
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValue;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::Result {
            ok: Box::new(NamedCLType::U128),
            err: Box::new(NamedCLType::String)
        });
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValue;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::List(Box::new(NamedCLType::Custom(
            "MyType".to_string()
        ))));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::List(Box::new(WasmType::Custom("MyType".to_string())));
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::List(Box::new(NamedCLType::Option(Box::new(
            NamedCLType::U128
        )))));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValueList;
        assert_eq!(wasm_type, expected);

        let ty = Type(NamedCLType::List(Box::new(NamedCLType::ByteArray(32))));
        let wasm_type = WasmType::from(&ty);
        let expected = WasmType::JsValueList;
        assert_eq!(wasm_type, expected);
    }
}
