//! XJasper WASM
//!
//! WASM bindings for XJasper.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn render_report(template_json: &str, data_json: &str) -> Result<Vec<u8>, JsValue> {
    Err(JsValue::from_str("Not implemented yet"))
}
