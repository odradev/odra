mod custom_types;
mod entry_points;

pub use custom_types::types_def;
pub use entry_points::client;
use quote::ToTokens;

use crate::types::{OdraType, WasmType};

impl PartialEq<OdraType> for WasmType {
    fn eq(&self, other: &OdraType) -> bool {
        self.to_token_stream().to_string() == other.to_token_stream().to_string()
    }
}

pub fn imports() -> proc_macro2::TokenStream {
    quote::quote! {
        #![allow(unused_imports)]
        use odra_wasm_client::wasm_bindgen as wasm_bindgen;
        use odra_wasm_client::wasm_bindgen_futures as wasm_bindgen_futures;
        use odra_wasm_client::JsValueSerdeExt;
        use wasm_bindgen::prelude::*;
    }
}
