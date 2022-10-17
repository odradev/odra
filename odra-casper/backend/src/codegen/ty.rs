use odra::types::CLType;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub(super) struct WrappedType<'a>(pub &'a CLType);

impl ToTokens for WrappedType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match &self.0 {
            CLType::Bool => quote!(casper_backend::backend::casper_types::CLType::Bool),
            CLType::I32 => quote!(casper_backend::backend::casper_types::CLType::I32),
            CLType::I64 => quote!(casper_backend::backend::casper_types::CLType::I64),
            CLType::U8 => quote!(casper_backend::backend::casper_types::CLType::U8),
            CLType::U32 => quote!(casper_backend::backend::casper_types::CLType::U32),
            CLType::U64 => quote!(casper_backend::backend::casper_types::CLType::U64),
            CLType::U128 => quote!(casper_backend::backend::casper_types::CLType::U128),
            CLType::U256 => quote!(casper_backend::backend::casper_types::CLType::U256),
            CLType::U512 => quote!(casper_backend::backend::casper_types::CLType::U512),
            CLType::Unit => quote!(casper_backend::backend::casper_types::CLType::Unit),
            CLType::String => quote!(casper_backend::backend::casper_types::CLType::String),
            CLType::Option(ty) => {
                let value_stream = WrappedType(&**ty).to_token_stream();
                quote!(casper_backend::backend::casper_types::CLType::Option(Box::new(#value_stream)))
            }
            CLType::Any => quote!(casper_backend::backend::casper_types::CLType::Any),
            CLType::Key => quote!(casper_backend::backend::casper_types::CLType::Key),
            CLType::URef => quote!(casper_backend::backend::casper_types::CLType::URef),
            CLType::PublicKey => quote!(casper_backend::backend::casper_types::CLType::PublicKey),
            CLType::List(ty) => {
                let value_stream = WrappedType(&**ty).to_token_stream();
                quote!(casper_backend::backend::casper_types::CLType::List(Box::new(#value_stream)))
            }
            CLType::ByteArray(bytes) => {
                quote!(casper_backend::backend::casper_types::CLType::ByteArray(#bytes))
            }
            CLType::Result { ok, err } => {
                let ok_stream = WrappedType(&**ok).to_token_stream();
                let err_stream = WrappedType(&**err).to_token_stream();
                quote! {
                    casper_backend::backend::casper_types::CLType::Result {
                        ok: Box::new(#ok_stream),
                        err: Box::new(#err_stream),
                    }
                }
            }
            CLType::Map { key, value } => {
                let key_stream = WrappedType(&**key).to_token_stream();
                let value_stream = WrappedType(&**value).to_token_stream();
                quote! {
                    casper_backend::backend::casper_types::CLType::Map {
                        key: Box::new(#key_stream),
                        value: Box::new(#value_stream),
                    }
                }
            }
            CLType::Tuple1(ty) => {
                let ty = &**ty.get(0).unwrap();
                let ty = WrappedType(ty).to_token_stream();
                quote! {
                    casper_backend::backend::casper_types::CLType::Tuple1([#ty])
                }
            }
            CLType::Tuple2(ty) => {
                let t1 = &**ty.get(0).unwrap();
                let t1 = WrappedType(t1).to_token_stream();
                let t2 = &**ty.get(1).unwrap();
                let t2 = WrappedType(t2).to_token_stream();
                quote! {
                    casper_backend::backend::casper_types::CLType::Tuple2([#t1, #t2])
                }
            }
            CLType::Tuple3(ty) => {
                let t1 = &**ty.get(0).unwrap();
                let t1 = WrappedType(t1).to_token_stream();
                let t2 = &**ty.get(1).unwrap();
                let t2 = WrappedType(t2).to_token_stream();
                let t3 = &**ty.get(2).unwrap();
                let t3 = WrappedType(t3).to_token_stream();
                quote! {
                    casper_backend::backend::casper_types::CLType::Tuple2([#t1, #t2, #t3])
                }
            }
        };
        tokens.extend(stream);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::assert_eq_tokens;

    #[test]
    fn test_simple_type() {
        let ty = CLType::Bool;
        let wrapped_type = WrappedType(&ty);
        assert_eq_tokens(
            wrapped_type,
            quote!(casper_backend::backend::casper_types::CLType::Bool),
        );
    }

    #[test]
    fn test_complex_type() {
        let ty = CLType::Option(Box::new(CLType::Tuple2([
            Box::new(CLType::Bool),
            Box::new(CLType::I32),
        ])));
        let wrapped_type = WrappedType(&ty);
        assert_eq_tokens(
            wrapped_type,
            quote!(casper_backend::backend::casper_types::CLType::Option(
                Box::new(casper_backend::backend::casper_types::CLType::Tuple2([
                    casper_backend::backend::casper_types::CLType::Bool,
                    casper_backend::backend::casper_types::CLType::I32
                ]))
            )),
        );
    }
}
