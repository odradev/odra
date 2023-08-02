use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {

    pub type CasperWalletProvider;

    #[wasm_bindgen]
    pub fn CasperWalletProvider() -> CasperWalletProvider;

    #[wasm_bindgen(method)]
    pub fn requestConnection(this: &CasperWalletProvider);

    #[wasm_bindgen(method)]
    pub fn signDeploy(this: &CasperWalletProvider);
}