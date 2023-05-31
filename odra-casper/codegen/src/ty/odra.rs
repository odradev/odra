use odra_types::Type;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct OdraType<'a>(&'a Type);

impl<'a> From<&'a Type> for OdraType<'a> {
    fn from(value: &'a Type) -> Self {
        OdraType(value)
    }
}

impl ToTokens for OdraType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match &self.0 {
            Type::Bool => quote!(bool),
            Type::I32 => quote!(i32),
            Type::I64 => quote!(i64),
            Type::U8 => quote!(u8),
            Type::U32 => quote!(u32),
            Type::U64 => quote!(u64),
            Type::U128 => quote!(odra::types::U128),
            Type::U256 => quote!(odra::types::U256),
            Type::U512 => quote!(odra::types::U512),
            Type::Unit => quote!(()),
            Type::String => quote!(String),
            Type::Address => quote!(odra::types::Address),
            Type::Option(ty) => {
                let ty = OdraType(ty);
                quote!(Option<#ty>)
            }
            Type::Any => quote!(Any),
            Type::Vec(ty) => {
                let ty = OdraType(ty);
                quote!(Vec<#ty>)
            }
            Type::Result { ok, err } => {
                let ok = OdraType(ok);
                let err = OdraType(err);
                quote!(Result<#ok, #err>)
            }
            Type::Map { key, value } => {
                let key = OdraType(key);
                let value = OdraType(value);
                quote!(std::collections::BTreeMap<#key, #value>)
            }
            Type::Tuple1(ty) => {
                let ty = OdraType(ty.get(0).unwrap());
                quote!((#ty,))
            }
            Type::Tuple2(ty) => {
                let t1 = OdraType(ty.get(0).unwrap());
                let t2 = OdraType(ty.get(1).unwrap());
                quote!((#t1, #t2))
            }
            Type::Tuple3(ty) => {
                let t1 = OdraType(ty.get(0).unwrap());
                let t2 = OdraType(ty.get(1).unwrap());
                let t3 = OdraType(ty.get(2).unwrap());
                quote!((#t1, #t2, #t3))
            }
            Type::ByteArray(b) => quote!([u8; #b]),
            Type::Slice(ty) => {
                let value = OdraType(ty);
                quote!(Vec<#value>)
            }
        };
        tokens.extend(stream);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use quote::ToTokens;

    #[test]
    fn test_bool() {
        let odra_type = OdraType(&Type::Bool);
        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "bool");
    }

    #[test]
    fn test_i32() {
        let odra_type = OdraType(&Type::I32);
        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "i32");
    }

    #[test]
    fn test_u256() {
        let odra_type = OdraType(&Type::U256);
        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "odra :: types :: U256");
    }

    #[test]
    fn test_option() {
        let ty = Type::Option(Box::new(Type::Bool));
        let odra_type = OdraType(&ty);
        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "Option < bool >");
    }

    #[test]
    fn test_vec() {
        let ty = Type::Vec(Box::new(Type::Bool));
        let odra_type = OdraType(&ty);
        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "Vec < bool >");
    }

    #[test]
    fn test_result() {
        let ty = Type::Result {
            ok: Box::new(Type::Bool),
            err: Box::new(Type::U8)
        };
        let odra_type = OdraType(&ty);
        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "Result < bool , u8 >");
    }

    #[test]
    fn test_map() {
        let ty = Type::Map {
            key: Box::new(Type::String),
            value: Box::new(Type::I32)
        };
        let odra_type = OdraType(&ty);

        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(
            tokens.to_string(),
            "std :: collections :: BTreeMap < String , i32 >"
        );
    }

    #[test]
    fn test_tuple1() {
        let ty = Type::Tuple1([Box::new(Type::I32)]);
        let odra_type = OdraType(&ty);

        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "(i32 ,)");
    }

    #[test]
    fn test_tuple2() {
        let ty = Type::Tuple2([Box::new(Type::I32), Box::new(Type::Bool)]);
        let odra_type = OdraType(&ty);
        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "(i32 , bool)");
    }

    #[test]
    fn test_tuple3() {
        let ty = Type::Tuple3([
            Box::new(Type::I32),
            Box::new(Type::Bool),
            Box::new(Type::U8)
        ]);
        let odra_type = OdraType(&ty);
        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "(i32 , bool , u8)");
    }

    #[test]
    fn test_byte_array() {
        let odra_type = OdraType(&Type::ByteArray(32));
        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "[u8 ; 32u32]");
    }

    #[test]
    fn test_slice() {
        let ty = Type::Slice(Box::new(Type::Bool));
        let odra_type = OdraType(&ty);
        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "Vec < bool >");
    }

    #[test]
    fn test_complex_option() {
        let ty = Type::Option(Box::new(Type::Vec(Box::new(Type::I32))));
        let odra_type = OdraType(&ty);

        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "Option < Vec < i32 > >");
    }

    #[test]
    fn test_complex_vec() {
        let ty = Type::Vec(Box::new(Type::Option(Box::new(Type::U8))));
        let odra_type = OdraType(&ty);

        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "Vec < Option < u8 > >");
    }

    #[test]
    fn test_complex_map() {
        let ty = Type::Map {
            key: Box::new(Type::String),
            value: Box::new(Type::Option(Box::new(Type::Vec(Box::new(Type::Bool)))))
        };
        let odra_type = OdraType(&ty);

        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(
            tokens.to_string(),
            "std :: collections :: BTreeMap < String , Option < Vec < bool > > >"
        );
    }

    #[test]
    fn test_complex_result() {
        let ty = Type::Result {
            ok: Box::new(Type::Vec(Box::new(Type::Option(Box::new(Type::I32))))),
            err: Box::new(Type::Map {
                key: Box::new(Type::String),
                value: Box::new(Type::Vec(Box::new(Type::Bool)))
            })
        };
        let odra_type = OdraType(&ty);

        let mut tokens = TokenStream::new();
        odra_type.to_tokens(&mut tokens);
        assert_eq!(tokens.to_string(), "Result < Vec < Option < i32 > > , std :: collections :: BTreeMap < String , Vec < bool > > >");
    }
}
