//! Async utils
use std::time::Duration;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};

use crate::utils::{debug_expect, debug_panic};

/// Sleeps for the given duration using js `setTimeout`.
pub async fn sleep(time: Duration) {
    let milis = if let Ok(milis) = i32::try_from(time.as_millis()) {
        milis
    } else {
        debug_panic!(
            "Sleep duration {}ms overflows `i32` (will use `i32::MAX` in release mode.)",
            time.as_millis()
        );
        i32::MAX
    };

    let (tx, rx) = futures_channel::oneshot::channel();
    let res = crate::get_window().set_timeout_with_callback_and_timeout_and_arguments_0(
        create_closure(tx).as_ref().unchecked_ref(),
        milis,
    );
    debug_expect!(
        res,
        "Failed to set timeout. This is a bug in the browser or the framework."
    );
    debug_expect!(rx.await, "Failed to receive timeout signal");
}

/// Wait for the next browser animation frame.
pub async fn next_animation_frame() {
    let (tx, rx) = futures_channel::oneshot::channel();

    let res =
        crate::get_window().request_animation_frame(create_closure(tx).as_ref().unchecked_ref());
    debug_expect!(
        res,
        "Failed to request animation frame. This is a bug in the browser or the framework."
    );
    debug_expect!(rx.await, "Failed to receive animation frame signal");
}

/// Convert a `tx` to a `Function` that sends a message once called.
/// `Closure::once_into_js` is used to ensure that the closure is dropped after it is called.
fn create_closure(tx: futures_channel::oneshot::Sender<()>) -> JsValue {
    Closure::once_into_js(move || {
        let _ = tx.send(());
    })
}
