use odra_types::CLType;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub struct CasperType<'a>(pub &'a CLType);

impl ToTokens for CasperType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let stream = match &self.0 {
            CLType::Bool => quote!(odra::types::CLType::Bool),
            CLType::I32 => quote!(odra::types::CLType::I32),
            CLType::I64 => quote!(odra::types::CLType::I64),
            CLType::U8 => quote!(odra::types::CLType::U8),
            CLType::U32 => quote!(odra::types::CLType::U32),
            CLType::U64 => quote!(odra::types::CLType::U64),
            CLType::U128 => quote!(odra::types::CLType::U128),
            CLType::U256 => quote!(odra::types::CLType::U256),
            CLType::U512 => quote!(odra::types::CLType::U512),
            CLType::Unit => quote!(odra::types::CLType::Unit),
            CLType::String => quote!(odra::types::CLType::String),
            CLType::Key => quote!(odra::types::CLType::Key),
            CLType::PublicKey => quote!(odra::types::CLType::PublicKey),
            CLType::Option(ty) => {
                let ty = CasperType(ty);
                quote!(odra::types::CLType::Option(odra::prelude::boxed::Box::new(#ty)))
            }
            CLType::Any => quote!(odra::types::CLType::Any),
            CLType::List(ty) => {
                let ty = CasperType(ty);
                quote!(odra::types::CLType::List(odra::prelude::boxed::Box::new(#ty)))
            }
            CLType::Result { ok, err } => {
                let ok = CasperType(ok);
                let err = CasperType(err);
                quote! {
                    odra::types::CLType::Result {
                        ok: odra::prelude::boxed::Box::new(#ok),
                        err: odra::prelude::boxed::Box::new(#err),
                    }
                }
            }
            CLType::Map { key, value } => {
                let key = CasperType(key);
                let value = CasperType(value);
                quote! {
                    odra::types::CLType::Map {
                        key: odra::prelude::boxed::Box::new(#key),
                        value: odra::prelude::boxed::Box::new(#value),
                    }
                }
            }
            CLType::Tuple1(ty) => {
                let ty = ty.get(0).unwrap();
                let ty = CasperType(ty);
                quote! {
                    odra::types::CLType::Tuple1([odra::prelude::boxed::Box::new(#ty)])
                }
            }
            CLType::Tuple2(ty) => {
                let t1 = ty.get(0).unwrap();
                let t1 = CasperType(t1);
                let t2 = ty.get(1).unwrap();
                let t2 = CasperType(t2);
                quote! {
                    odra::types::CLType::Tuple2([odra::prelude::boxed::Box::new(#t1), odra::prelude::boxed::Box::new(#t2)])
                }
            }
            CLType::Tuple3(ty) => {
                let t1 = ty.get(0).unwrap();
                let t1 = CasperType(t1);
                let t2 = ty.get(1).unwrap();
                let t2 = CasperType(t2);
                let t3 = ty.get(2).unwrap();
                let t3 = CasperType(t3);
                quote! {
                    odra::types::CLType::Tuple2([odra::prelude::boxed::Box::new(#t1), odra::prelude::boxed::Box::new(#t2), odra::prelude::boxed::Box::new(#t3)])
                }
            }
            CLType::ByteArray(b) => quote!(odra::types::CLType::ByteArray(#b)),
            CLType::URef => quote!(odra::types::CLType::URef)
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
        let ty = CLType::Bool;
        let wrapped_type = CasperType(&ty);
        assert_eq_tokens(wrapped_type, quote!(odra::types::CLType::Bool));
    }

    #[test]
    fn test_complex_type() {
        let ty = CLType::Option(Box::new(CLType::Tuple2([
            Box::new(CLType::Bool),
            Box::new(CLType::I32)
        ])));
        let wrapped_type = CasperType(&ty);
        assert_eq_tokens(
            wrapped_type,
            quote!(odra::types::CLType::Option(odra::prelude::boxed::Box::new(
                odra::types::CLType::Tuple2([
                    odra::prelude::boxed::Box::new(odra::types::CLType::Bool),
                    odra::prelude::boxed::Box::new(odra::types::CLType::I32)
                ])
            )))
        );
    }
}
