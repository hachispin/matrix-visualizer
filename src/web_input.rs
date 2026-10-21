//! Native touch editing for canvas number fields. Focus starts in the browser's tap handler
//! so mobile browsers can open their software keyboard without an asynchronous focus request.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/web_input.js")]
extern "C" {
    pub fn begin_frame(width: f32, height: f32);
    pub fn end_frame();
    pub fn is_editing(id: &str) -> bool;
    pub fn take_value(id: &str) -> Option<String>;
    pub fn register_number(id: &str, value: &str, label: &str, bounds: &[f32]);
}
