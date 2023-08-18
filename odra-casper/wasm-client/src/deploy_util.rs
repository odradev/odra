use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = du)]
    pub fn deployFromJson(json: JsValue) -> JsValue;
}
