use convert_case::{Case, Casing};
use odra_schema::casper_contract_schema::{NamedCLType, StructMember, Type};
use quote::{format_ident, ToTokens};
use syn::parse_quote;

pub enum OdraType {
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
    Key,
    URef,
    PublicKey,
    Option(Box<OdraType>),
    List(Box<OdraType>),
    ByteArray(u32),
    Result(Box<OdraType>, Box<OdraType>),
    Map(Box<OdraType>, Box<OdraType>),
    Tuple1(Box<OdraType>),
    Tuple2(Box<OdraType>, Box<OdraType>),
    Tuple3(Box<OdraType>, Box<OdraType>, Box<OdraType>),
    Custom(String)
}

impl OdraType {
    pub fn is_cloneable(&self) -> bool {
        match self {
            OdraType::Option(e) if e.is_cloneable() => true,
            OdraType::List(e) if e.is_cloneable() => true,
            OdraType::Result(ok, err) if ok.is_cloneable() && err.is_cloneable() => true,
            OdraType::String
            | OdraType::ByteArray(_)
            | OdraType::Bool
            | OdraType::I32
            | OdraType::I64
            | OdraType::U8
            | OdraType::U32
            | OdraType::U64
            | OdraType::Custom(_) => true,
            _ => false
        }
    }

    pub fn field(&self, member: &StructMember) -> syn::Field {
        let field_name = format_ident!("{}", member.name);
        let js_name = member.name.to_case(Case::Camel);
        if self.is_cloneable() {
            parse_quote!(#[wasm_bindgen(js_name = #js_name)] pub #field_name: #self)
        } else {
            parse_quote!(#[wasm_bindgen(js_name = #js_name)] #field_name: #self)
        }
    }
}

impl ToTokens for OdraType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = match self {
            OdraType::Bool => quote::quote!(bool),
            OdraType::I32 => quote::quote!(i32),
            OdraType::I64 => quote::quote!(i64),
            OdraType::U8 => quote::quote!(u8),
            OdraType::U32 => quote::quote!(u32),
            OdraType::U64 => quote::quote!(u64),
            OdraType::U128 => quote::quote!(odra_wasm_client::casper_types::U128),
            OdraType::U256 => quote::quote!(odra_wasm_client::casper_types::U256),
            OdraType::U512 => quote::quote!(odra_wasm_client::casper_types::U512),
            OdraType::Unit => quote::quote!(()),
            OdraType::String => quote::quote!(String),
            OdraType::Key => quote::quote!(odra_wasm_client::OdraAddress),
            OdraType::URef => quote::quote!(odra_wasm_client::casper_types::URef),
            OdraType::PublicKey => quote::quote!(odra_wasm_client::casper_types::PublicKey),
            OdraType::Option(inner) => {
                let inner = inner.to_token_stream();
                quote::quote!(Option<#inner>)
            }
            OdraType::List(inner) => {
                quote::quote!(Vec<#inner>)
            }
            OdraType::ByteArray(_n) => {
                quote::quote!(Vec<u8>)
            }
            OdraType::Result(ok, err) => {
                quote::quote!(Result<#ok, #err>)
            }
            OdraType::Map(key, value) => {
                quote::quote!(std::collections::BTreeMap<#key, #value>)
            }
            OdraType::Tuple1(inner) => {
                let inner = inner.to_token_stream();
                quote::quote!( (#inner,) )
            }
            OdraType::Tuple2(inner1, inner2) => {
                quote::quote!( (#inner1, #inner2) )
            }
            OdraType::Tuple3(inner1, inner2, inner3) => {
                quote::quote!( (#inner1, #inner2, #inner3) )
            }
            OdraType::Custom(name) => {
                let ident = format_ident!("{}", name);
                quote::quote!(#ident)
            }
        };
        tokens.extend(ty);
    }
}

impl From<&Type> for OdraType {
    fn from(ty: &Type) -> Self {
        named_cl_type_to_odra_type(ty)
    }
}

fn named_cl_type_to_odra_type(ty: &Type) -> OdraType {
    match &ty.0 {
        NamedCLType::Bool => OdraType::Bool,
        NamedCLType::I32 => OdraType::I32,
        NamedCLType::I64 => OdraType::I64,
        NamedCLType::U8 => OdraType::U8,
        NamedCLType::U32 => OdraType::U32,
        NamedCLType::U64 => OdraType::U64,
        NamedCLType::U128 => OdraType::U128,
        NamedCLType::U256 => OdraType::U256,
        NamedCLType::U512 => OdraType::U512,
        NamedCLType::Unit => OdraType::Unit,
        NamedCLType::String => OdraType::String,
        NamedCLType::Key => OdraType::Key,
        NamedCLType::URef => OdraType::URef,
        NamedCLType::PublicKey => OdraType::PublicKey,
        NamedCLType::Option(named_cltype) => {
            let inner = named_cl_type_to_odra_type(&Type(*named_cltype.clone()));
            OdraType::Option(Box::new(inner))
        }
        NamedCLType::List(named_cltype) => {
            let inner = named_cl_type_to_odra_type(&Type(*named_cltype.clone()));
            OdraType::List(Box::new(inner))
        }
        NamedCLType::ByteArray(n) => OdraType::ByteArray(*n),
        NamedCLType::Result { ok, err } => {
            let ok = named_cl_type_to_odra_type(&Type(*ok.clone()));
            let err = named_cl_type_to_odra_type(&Type(*err.clone()));
            OdraType::Result(Box::new(ok), Box::new(err))
        }
        NamedCLType::Map { key, value } => {
            let key = named_cl_type_to_odra_type(&Type(*key.clone()));
            let value = named_cl_type_to_odra_type(&Type(*value.clone()));
            OdraType::Map(Box::new(key), Box::new(value))
        }
        NamedCLType::Tuple1(ty) => {
            let inner = named_cl_type_to_odra_type(&Type(*ty[0].clone()));
            OdraType::Tuple1(Box::new(inner))
        }
        NamedCLType::Tuple2(ty) => {
            let inner = named_cl_type_to_odra_type(&Type(*ty[0].clone()));
            let inner2 = named_cl_type_to_odra_type(&Type(*ty[1].clone()));
            OdraType::Tuple2(Box::new(inner), Box::new(inner2))
        }
        NamedCLType::Tuple3(ty) => {
            let inner = named_cl_type_to_odra_type(&Type(*ty[0].clone()));
            let inner2 = named_cl_type_to_odra_type(&Type(*ty[1].clone()));
            let inner3 = named_cl_type_to_odra_type(&Type(*ty[2].clone()));
            OdraType::Tuple3(Box::new(inner), Box::new(inner2), Box::new(inner3))
        }
        NamedCLType::Custom(name) => OdraType::Custom(name.to_owned())
    }
}
