use js_sys::Promise;
use wasm_bindgen::prelude::*;

use crate::cspr_click::types::AccountInfo;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = signIn, catch)]
    pub fn sign_in() -> Result<(), JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = connect, catch)]
    pub fn connect(provider: &str) -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = disconnect, catch)]
    pub fn disconnect() -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = forgetAccount, catch)]
    pub fn forget_account(account: AccountInfo) -> Result<(), JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = getActiveAccountAsync, catch)]
    pub fn get_active_account(options: &JsValue) -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = getActivePublicKey, catch)]
    pub fn get_active_public_key() -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], catch, js_name = "on")]
    pub(crate) fn on_csprclick_event(
        event: &str,
        callback: &js_sys::Function
    ) -> Result<(), JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = "isUnlocked", catch)]
    pub fn is_unlocked(provider: &str) -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = send, catch)]
    pub fn send(
        transaction: &str,
        signing_public_key: &str,
        on_status_update: &js_sys::Function
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = sign, catch)]
    pub(crate) fn sign(transaction: &str, signing_public_key: &str) -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = signOut, catch)]
    pub fn sign_out() -> Result<(), JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = signInWithAccount, catch)]
    pub fn sign_in_with_account(account: AccountInfo) -> Result<Promise, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "csprclick"], js_name = switchAccount, catch)]
    pub fn switch_account() -> Result<Promise, JsValue>;
}
