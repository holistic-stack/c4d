use wasm_bindgen::prelude::*;
use manifold_rs::process_openscad;

#[wasm_bindgen]
pub fn hello_world() -> String {
	"Hello from Rust WASM!".to_string()
}

#[wasm_bindgen]
pub fn compile_openscad(source: &str) -> Result<Vec<f64>, JsValue> {
    process_openscad(source).map_err(|e| JsValue::from_str(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_cube() {
        let source = "cube(10);";
        let result = compile_openscad(source);
        assert!(result.is_ok());
        let vertices = result.unwrap();
        
        // Cube has 6 faces.
        // Each face is triangulated.
        // If 2 triangles per face -> 12 triangles.
        // 3 vertices per triangle -> 36 vertices.
        // Each vertex has 3 coords -> 108 floats.
        
        // Let's assert we have some vertices.
        assert!(vertices.len() > 0);
        assert_eq!(vertices.len() % 9, 0); // Multiple of 3 vertices * 3 coords
    }
}
