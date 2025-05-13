//! Async utils
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_channel::mpsc::UnboundedReceiver;
use futures_core::stream::Stream;
use futures_util::stream::StreamExt;
use futures_util::task::noop_waker;
use wasm_bindgen::prelude::Closure;
use web_sys::js_sys::Function;

use crate::utils::{DropGuard, debug_panic};

/// Sleeps for the given duration using js `setTimeout`.
///
/// Even tho this api takes a Duration it needs to be cast to a i32 of milliseconds to be passed to
/// the js apis.
/// If this conversion overflow it will produce a panic in debug mode.
/// In release mode it will use `i32::MAX` (around 24 days) as the duration.
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
    let res = crate::get_window()
        .set_timeout_with_callback_and_timeout_and_arguments_0(&create_closure(tx), milis);
    let Ok(timeout_id) = res else {
        debug_panic!("Failed to set timeout. This is a bug in the browser or the framework.");
        return;
    };

    let mut drop_guard = DropGuard::new(move || {
        crate::get_window().clear_timeout_with_handle(timeout_id);
    });
    if rx.await.is_err() {
        debug_panic!("Failed to receive timeout signal");
        return;
    }
    drop_guard.cancel();
}

/// Wait for the next browser animation frame.
pub async fn next_animation_frame() {
    let (tx, rx) = futures_channel::oneshot::channel();

    let res = crate::get_window().request_animation_frame(&create_closure(tx));
    let Ok(frame_id) = res else {
        debug_panic!(
            "Failed to request animation frame. This is a bug in the browser or the framework."
        );
        return;
    };

    let mut drop_guard = DropGuard::new(move || {
        let res = crate::get_window().cancel_animation_frame(frame_id);
        if res.is_err() {
            debug_panic!(
                "Failed to cancel animation frame. This is a bug in the browser or the framework."
            );
        }
    });
    if rx.await.is_err() {
        debug_panic!("Failed to receive animation frame signal");
        return;
    }
    drop_guard.cancel();
}

/// Convert a `tx` to a `Function` that sends a message once called.
/// `Closure::once_into_js` is used to ensure that the closure is dropped after it is called.
fn create_closure(tx: futures_channel::oneshot::Sender<()>) -> Function {
    Closure::once_into_js(move || {
        let _ = tx.send(());
    })
    .into()
}

/// Wait for at least one mesassge on the channel
/// And then return all of the available messages
/// Returns `None` if the channel is closed
pub(crate) async fn drain_available<T>(rx: &mut UnboundedReceiver<T>) -> Option<Vec<T>> {
    let first = rx.next().await?;

    let mut messages = Vec::with_capacity(4);
    messages.push(first);

    // Create a waker that will never wake anything
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    // Drain all messages that are immediately available
    while let Poll::Ready(Some(msg)) = Pin::new(&mut *rx).poll_next(&mut cx) {
        messages.push(msg);
    }

    Some(messages)
}
