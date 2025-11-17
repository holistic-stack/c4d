use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hello_world() -> String {
	"Hello from Rust WASM!".to_string()
}
