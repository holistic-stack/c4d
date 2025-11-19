use wasm_bindgen::prelude::*;
use js_sys::Float64Array;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn compile_and_render(source: &str) -> Result<Float64Array, JsValue> {
    init_panic_hook();
    
    // log(&format!("Received source code: {}", source));
    
    match manifold_rs::compile(source) {
        Ok(vertices) => {
            // Convert Vec<f64> to Float64Array
            let array = Float64Array::from(&vertices[..]);
            Ok(array)
        },
        Err(e) => {
            // Convert KernelError to TraceDiagnostic list and serialize to JSON string
            let diagnostics = manifold_rs::error_to_trace(&e, source);
            let json = serde_json::to_string(&diagnostics).map_err(|e| JsValue::from_str(&format!("JSON error: {}", e)))?;
            Err(JsValue::from_str(&json))
        }
    }
}
