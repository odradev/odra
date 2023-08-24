use odra_types::Type;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct CasperType<'a>(pub &'a Type);

impl ToTokens for CasperType<'_> {
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
            Type::Address => quote!(odra::casper::casper_types::CLType::Key),
            Type::PublicKey => quote!(odra::casper::casper_types::CLType::PublicKey),
            Type::Option(ty) => {
                let ty = CasperType(ty);
                quote!(odra::casper::casper_types::CLType::Option(alloc::boxed::Box::new(#ty)))
            }
            Type::Any => quote!(odra::casper::casper_types::CLType::Any),
            Type::Vec(ty) => {
                let ty = CasperType(ty);
                quote!(odra::casper::casper_types::CLType::List(alloc::boxed::Box::new(#ty)))
            }
            Type::Result { ok, err } => {
                let ok = CasperType(ok);
                let err = CasperType(err);
                quote! {
                    odra::casper::casper_types::CLType::Result {
                        ok: alloc::boxed::Box::new(#ok),
                        err: alloc::boxed::Box::new(#err),
                    }
                }
            }
            Type::Map { key, value } => {
                let key = CasperType(key);
                let value = CasperType(value);
                quote! {
                    odra::casper::casper_types::CLType::Map {
                        key: alloc::boxed::Box::new(#key),
                        value: alloc::boxed::Box::new(#value),
                    }
                }
            }
            Type::Tuple1(ty) => {
                let ty = ty.get(0).unwrap();
                let ty = CasperType(ty);
                quote! {
                    odra::casper::casper_types::CLType::Tuple1([alloc::boxed::Box::new(#ty)])
                }
            }
            Type::Tuple2(ty) => {
                let t1 = ty.get(0).unwrap();
                let t1 = CasperType(t1);
                let t2 = ty.get(1).unwrap();
                let t2 = CasperType(t2);
                quote! {
                    odra::casper::casper_types::CLType::Tuple2([alloc::boxed::Box::new(#t1), alloc::boxed::Box::new(#t2)])
                }
            }
            Type::Tuple3(ty) => {
                let t1 = ty.get(0).unwrap();
                let t1 = CasperType(t1);
                let t2 = ty.get(1).unwrap();
                let t2 = CasperType(t2);
                let t3 = ty.get(2).unwrap();
                let t3 = CasperType(t3);
                quote! {
                    odra::casper::casper_types::CLType::Tuple2([alloc::boxed::Box::new(#t1), alloc::boxed::Box::new(#t2), alloc::boxed::Box::new(#t3)])
                }
            }
            Type::ByteArray(b) => quote!(odra::casper::casper_types::CLType::ByteArray(#b)),
            Type::Slice(ty) => {
                let ty = CasperType(ty);
                quote!(odra::casper::casper_types::CLType::List(alloc::boxed::Box::new(#ty)))
            }
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
        let wrapped_type = CasperType(&ty);
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
        let wrapped_type = CasperType(&ty);
        assert_eq_tokens(
            wrapped_type,
            quote!(odra::casper::casper_types::CLType::Option(
                alloc::boxed::Box::new(odra::casper::casper_types::CLType::Tuple2([
                    alloc::boxed::Box::new(odra::casper::casper_types::CLType::Bool),
                    alloc::boxed::Box::new(odra::casper::casper_types::CLType::I32)
                ]))
            ))
        );
    }
}
