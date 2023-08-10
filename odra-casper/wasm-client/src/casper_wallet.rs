use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {

    pub type CasperWalletProvider;

    #[wasm_bindgen]
    pub fn CasperWalletProvider() -> CasperWalletProvider;

    #[wasm_bindgen(method)]
    pub async fn requestConnection(this: &CasperWalletProvider);

    #[wasm_bindgen(method)]
    pub async fn sign(
        this: &CasperWalletProvider,
        deployJson: String,
        signingPublicKeyHex: String
    ) -> JsValue;

    #[wasm_bindgen(method)]
    pub async fn getActivePublicKey(this: &CasperWalletProvider) -> JsValue;
}
