use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// deployParam: DeployHeader
    /// session: ExecutableDeployItem
    /// payment: ExecutableDeployItem
    pub async fn signDeploy(
        publicKeyHex: String,
        deployParam: JsValue,
        session: JsValue,
        payment: JsValue
    );
}
