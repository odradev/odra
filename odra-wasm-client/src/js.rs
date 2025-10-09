use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    pub fn warn(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn error(s: &str);
}
