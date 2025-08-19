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

    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}