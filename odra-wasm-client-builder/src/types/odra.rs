use odra_schema::casper_contract_schema::{NamedCLType, Type};
use quote::{format_ident, ToTokens};

pub struct OdraType(NamedCLType);

impl OdraType {
    pub fn is_copyable(&self) -> bool {
        match self.0.clone() {
            NamedCLType::Option(e) if OdraType(*e.clone()).is_copyable() => true,
            NamedCLType::List(e) if OdraType(*e.clone()).is_copyable() => true,
            NamedCLType::Result { ok, err }
                if OdraType(*ok.clone()).is_copyable() && OdraType(*err.clone()).is_copyable() =>
            {
                true
            }
            NamedCLType::String
            | NamedCLType::ByteArray(_)
            | NamedCLType::Bool
            | NamedCLType::I32
            | NamedCLType::I64
            | NamedCLType::U8
            | NamedCLType::U32
            | NamedCLType::U64
            | NamedCLType::Custom(_) => true,
            _ => false
        }
    }
}

impl ToTokens for OdraType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = match self.0.clone() {
            NamedCLType::Bool => quote::quote!(bool),
            NamedCLType::I32 => quote::quote!(i32),
            NamedCLType::I64 => quote::quote!(i64),
            NamedCLType::U8 => quote::quote!(u8),
            NamedCLType::U32 => quote::quote!(u32),
            NamedCLType::U64 => quote::quote!(u64),
            NamedCLType::U128 => quote::quote!(odra_wasm_client::casper_types::U128),
            NamedCLType::U256 => quote::quote!(odra_wasm_client::casper_types::U256),
            NamedCLType::U512 => quote::quote!(odra_wasm_client::casper_types::U512),
            NamedCLType::Unit => quote::quote!(()),
            NamedCLType::String => quote::quote!(String),
            NamedCLType::Key => quote::quote!(odra_wasm_client::OdraAddress),
            NamedCLType::URef => quote::quote!(odra_wasm_client::casper_types::URef),
            NamedCLType::PublicKey => quote::quote!(odra_wasm_client::casper_types::PublicKey),
            NamedCLType::Option(inner) => {
                let inner = OdraType(*inner);
                quote::quote!(Option<#inner>)
            }
            NamedCLType::List(inner) => {
                let inner = OdraType(*inner);
                quote::quote!(Vec<#inner>)
            }
            NamedCLType::ByteArray(_n) => {
                quote::quote!(Vec<u8>)
            }
            NamedCLType::Result { ok, err } => {
                let ok = OdraType(*ok);
                let err = OdraType(*err);
                quote::quote!(Result<#ok, #err>)
            }
            NamedCLType::Map { key, value } => {
                let key = OdraType(*key);
                let value = OdraType(*value);
                quote::quote!(std::collections::BTreeMap<#key, #value>)
            }
            NamedCLType::Tuple1(inner) => {
                let i1 = OdraType(*inner[0].clone());
                quote::quote!( (#i1,) )
            }
            NamedCLType::Tuple2(inner) => {
                let i1 = OdraType(*inner[0].clone());
                let i2 = OdraType(*inner[1].clone());
                quote::quote!( (#i1, #i2) )
            }
            NamedCLType::Tuple3(inner) => {
                let i1 = OdraType(*inner[0].clone());
                let i2 = OdraType(*inner[1].clone());
                let i3 = OdraType(*inner[2].clone());
                quote::quote!( (#i1, #i2, #i3) )
            }
            NamedCLType::Custom(name) => {
                let ident = format_ident!("{}", name);
                quote::quote!(#ident)
            }
        };
        tokens.extend(ty);
    }
}

impl From<&Type> for OdraType {
    fn from(ty: &Type) -> Self {
        OdraType(ty.0.clone())
    }
}
