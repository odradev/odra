pub mod attr;
pub mod expr;
pub mod ident;
pub mod member;
pub mod misc;
pub mod stmt;
pub mod string;
pub mod syn;
pub mod ty;

pub trait IntoCode {
    fn into_code(self) -> proc_macro::TokenStream;
}

impl<T: quote::ToTokens> IntoCode for Result<T, ::syn::Error> {
    fn into_code(self) -> proc_macro::TokenStream {
        match self {
            Ok(data) => data.to_token_stream(),
            Err(e) => e.to_compile_error()
        }
        .into()
    }
}
