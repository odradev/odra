use js_sys::Promise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub(crate) type CasperWalletProvider;

    #[wasm_bindgen(js_name = CasperWalletProvider)]
    pub(crate) fn casper_wallet_provider() -> CasperWalletProvider;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn requestConnection(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn sign(
        this: &CasperWalletProvider,
        deploy: &str,
        signing_public_key_hex: &str
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn signMessage(
        this: &CasperWalletProvider,
        message: &str,
        signing_public_key_hex: &str
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn requestSwitchAccount(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn disconnectFromSite(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn isConnected(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn getActivePublicKey(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_class = "CasperWalletProvider")]
    pub(crate) fn getVersion(this: &CasperWalletProvider) -> Result<Promise, JsValue>;

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
