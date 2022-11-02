use odra_types::Type;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub(super) struct WrappedType<'a>(pub &'a Type);

impl ToTokens for WrappedType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match &self.0 {
            Type::Bool => quote!(odra::casper::casper_types::CLType::Bool),
            Type::I32 => quote!(odra::casper::casper_types::CLType::I32),
            Type::I64 => quote!(odra::casper::casper_types::CLType::I64),
            Type::U8 => quote!(odra::casper::casper_types::CLType::U8),
            Type::U32 => quote!(odra::casper::casper_types::CLType::U32),
            Type::U64 => quote!(odra::casper::casper_types::CLType::U64),
            Type::U128 => quote!(odra::casper::casper_types::CLType::U128),
            Type::U256 => quote!(odra::casper::casper_types::CLType::U256),
            Type::U512 => quote!(odra::casper::casper_types::CLType::U512),
            Type::Unit => quote!(odra::casper::casper_types::CLType::Unit),
            Type::String => quote!(odra::casper::casper_types::CLType::String),
            Type::Option(ty) => {
                let value_stream = WrappedType(&**ty).to_token_stream();
                quote!(odra::casper::casper_types::CLType::Option(Box::new(#value_stream)))
            }
            Type::Any => quote!(odra::casper::casper_types::CLType::Any),
            Type::Vec(ty) => {
                let value_stream = WrappedType(&**ty).to_token_stream();
                quote!(odra::casper::casper_types::CLType::List(Box::new(#value_stream)))
            }
            Type::Result { ok, err } => {
                let ok_stream = WrappedType(&**ok).to_token_stream();
                let err_stream = WrappedType(&**err).to_token_stream();
                quote! {
                    odra::casper::casper_types::CLType::Result {
                        ok: Box::new(#ok_stream),
                        err: Box::new(#err_stream),
                    }
                }
            }
            Type::Map { key, value } => {
                let key_stream = WrappedType(&**key).to_token_stream();
                let value_stream = WrappedType(&**value).to_token_stream();
                quote! {
                    odra::casper::casper_types::CLType::Map {
                        key: Box::new(#key_stream),
                        value: Box::new(#value_stream),
                    }
                }
            }
            Type::Tuple1(ty) => {
                let ty = &**ty.get(0).unwrap();
                let ty = WrappedType(ty).to_token_stream();
                quote! {
                    odra::casper::casper_types::CLType::Tuple1([#ty])
                }
            }
            Type::Tuple2(ty) => {
                let t1 = &**ty.get(0).unwrap();
                let t1 = WrappedType(t1).to_token_stream();
                let t2 = &**ty.get(1).unwrap();
                let t2 = WrappedType(t2).to_token_stream();
                quote! {
                    odra::casper::casper_types::CLType::Tuple2([#t1, #t2])
                }
            }
            Type::Tuple3(ty) => {
                let t1 = &**ty.get(0).unwrap();
                let t1 = WrappedType(t1).to_token_stream();
                let t2 = &**ty.get(1).unwrap();
                let t2 = WrappedType(t2).to_token_stream();
                let t3 = &**ty.get(2).unwrap();
                let t3 = WrappedType(t3).to_token_stream();
                quote! {
                    odra::casper::casper_types::CLType::Tuple2([#t1, #t2, #t3])
                }
            }
            _ => quote!(odra::casper::casper_types::CLType::Any)
        };
        tokens.extend(stream);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_eq_tokens;

    #[test]
    fn test_simple_type() {
        let ty = Type::Bool;
        let wrapped_type = WrappedType(&ty);
        assert_eq_tokens(
            wrapped_type,
            quote!(odra::casper::casper_types::CLType::Bool)
        );
    }

    #[test]
    fn test_complex_type() {
        let ty = Type::Option(Box::new(Type::Tuple2([
            Box::new(Type::Bool),
            Box::new(Type::I32)
        ])));
        let wrapped_type = WrappedType(&ty);
        assert_eq_tokens(
            wrapped_type,
            quote!(odra::casper::casper_types::CLType::Option(Box::new(
                odra::casper::casper_types::CLType::Tuple2([
                    odra::casper::casper_types::CLType::Bool,
                    odra::casper::casper_types::CLType::I32
                ])
            )))
        );
    }
}
