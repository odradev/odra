use derive_more::From;
use quote::ToTokens;

use crate::GenerateCode;

#[derive(From)]
pub struct ContractStruct<'a> {
    contract: &'a odra_ir::ContractStruct,
}

impl GenerateCode for ContractStruct<'_> {
    fn generate_code(&self) -> proc_macro2::TokenStream {
        self.contract.to_token_stream()
    }
}
