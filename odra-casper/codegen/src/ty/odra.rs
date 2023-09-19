use odra_types::casper_types::CLType;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct OdraType<'a>(&'a CLType);

impl<'a> From<&'a CLType> for OdraType<'a> {
    fn from(value: &'a CLType) -> Self {
        OdraType(value)
    }
}

impl ToTokens for OdraType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match &self.0 {
            CLType::Bool => quote!(bool),
            CLType::I32 => quote!(i32),
            CLType::I64 => quote!(i64),
            CLType::U8 => quote!(u8),
            CLType::U32 => quote!(u32),
            CLType::U64 => quote!(u64),
            CLType::U128 => quote!(odra::types::casper_types::U128),
            CLType::U256 => quote!(odra::types::casper_types::U256),
            CLType::U512 => quote!(odra::types::casper_types::U512),
            CLType::Unit => quote!(()),
            CLType::String => quote!(odra::prelude::string::String),
            CLType::Key => quote!(odra::types::Address),
            CLType::PublicKey => quote!(odra::types::PublicKey),
            CLType::Option(ty) => {
                let ty = OdraType(ty);
                quote!(Option<#ty>)
            }
            CLType::Any => quote!(Any),
            CLType::List(ty) => {
                let ty = OdraType(ty);
                quote!(odra::prelude::vec::Vec<#ty>)
            }
            CLType::Result { ok, err } => {
                let ok = OdraType(ok);
                let err = OdraType(err);
                quote!(Result<#ok, #err>)
            }
            CLType::Map { key, value } => {
                let key = OdraType(key);
                let value = OdraType(value);
                quote!(odra::prelude::collections::BTreeMap<#key, #value>)
            }
            CLType::Tuple1(ty) => {
                let ty = OdraType(ty.get(0).unwrap());
                quote!((#ty,))
            }
            CLType::Tuple2(ty) => {
                let t1 = OdraType(ty.get(0).unwrap());
                let t2 = OdraType(ty.get(1).unwrap());
                quote!((#t1, #t2))
            }
            CLType::Tuple3(ty) => {
                let t1 = OdraType(ty.get(0).unwrap());
                let t2 = OdraType(ty.get(1).unwrap());
                let t3 = OdraType(ty.get(2).unwrap());
                quote!((#t1, #t2, #t3))
            }
            CLType::ByteArray(b) => quote!([u8; #b]),
            CLType::URef => todo!()
        };
        tokens.extend(stream);
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_eq_tokens;

    use super::*;

    #[test]
    fn test_bool() {
        let odra_type = OdraType(&CLType::Bool);
        let expected = quote!(bool);
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_i32() {
        let odra_type = OdraType(&CLType::I32);
        let expected = quote!(i32);
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_u256() {
        let odra_type = OdraType(&CLType::U256);
        let expected = quote!(odra::types::casper_types::U256);
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_option() {
        let ty = CLType::Option(Box::new(CLType::Bool));
        let odra_type = OdraType(&ty);
        let expected = quote!(Option<bool>);
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_vec() {
        let ty = CLType::List(Box::new(CLType::Bool));
        let odra_type = OdraType(&ty);
        let expected = quote!(odra::prelude::vec::Vec<bool>);
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_result() {
        let ty = CLType::Result {
            ok: Box::new(CLType::Bool),
            err: Box::new(CLType::U8)
        };
        let odra_type = OdraType(&ty);
        let expected = quote!(Result<bool, u8>);
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_map() {
        let ty = CLType::Map {
            key: Box::new(CLType::String),
            value: Box::new(CLType::I32)
        };
        let odra_type = OdraType(&ty);
        let expected = quote!(odra::prelude::collections::BTreeMap<odra::prelude::string::String, i32>);
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_tuple1() {
        let ty = CLType::Tuple1([Box::new(CLType::I32)]);
        let odra_type = OdraType(&ty);
        let expected = quote!((i32,));
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_tuple2() {
        let ty = CLType::Tuple2([Box::new(CLType::I32), Box::new(CLType::Bool)]);
        let odra_type = OdraType(&ty);
        let expected = quote!((i32, bool));
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_tuple3() {
        let ty = CLType::Tuple3([
            Box::new(CLType::I32),
            Box::new(CLType::Bool),
            Box::new(CLType::U8)
        ]);
        let odra_type = OdraType(&ty);
        let expected = quote!((i32, bool, u8));
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_byte_array() {
        let odra_type = OdraType(&CLType::ByteArray(32));
        let expected = quote!([u8; 32u32]);
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_complex_option() {
        let ty = CLType::Option(Box::new(CLType::List(Box::new(CLType::I32))));
        let odra_type = OdraType(&ty);

        let expected = quote!(Option<odra::prelude::vec::Vec<i32>>);

        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_complex_vec() {
        let ty = CLType::List(Box::new(CLType::Option(Box::new(CLType::U8))));
        let odra_type = OdraType(&ty);
        let expected = quote!(odra::prelude::vec::Vec<Option<u8>>);
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_complex_map() {
        let ty = CLType::Map {
            key: Box::new(CLType::String),
            value: Box::new(CLType::Option(Box::new(CLType::List(Box::new(
                CLType::Bool
            )))))
        };
        let odra_type = OdraType(&ty);
        let expected = quote!(odra::prelude::collections::BTreeMap< odra::prelude::string::String, Option< odra::prelude::vec::Vec< bool > > >);
        assert_eq_tokens(odra_type, expected);
    }

    #[test]
    fn test_complex_result() {
        let ty = CLType::Result {
            ok: Box::new(CLType::List(Box::new(CLType::Option(Box::new(
                CLType::I32
            ))))),
            err: Box::new(CLType::Map {
                key: Box::new(CLType::String),
                value: Box::new(CLType::List(Box::new(CLType::Bool)))
            })
        };
        let odra_type = OdraType(&ty);
        let expected =
            quote!(Result<odra::prelude::vec::Vec<Option<i32>>, odra::prelude::collections::BTreeMap<odra::prelude::string::String, odra::prelude::vec::Vec<bool>>>);
        assert_eq_tokens(odra_type, expected);
    }
}
