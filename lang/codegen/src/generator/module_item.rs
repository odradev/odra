use derive_more::From;
use quote::ToTokens;

use crate::GenerateCode;

#[derive(From)]
pub struct ModuleStruct<'a> {
    contract: &'a odra_ir::ModuleStruct,
}

impl GenerateCode for ModuleStruct<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        self.contract.to_token_stream()
    }
}
