//! Wrappers for js apis

/// Log the given string to browser console
pub fn log(msg: &str) {
    let msg = wasm_bindgen::JsValue::from_str(msg);
    web_sys::console::log_1(&msg);
}

