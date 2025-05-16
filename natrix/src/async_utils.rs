//! Variosu async versions of js callback apis
use std::any::Any;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_channel::mpsc::UnboundedReceiver;
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
    let (closure, function) = create_closure(tx);
    let res =
        crate::get_window().set_timeout_with_callback_and_timeout_and_arguments_0(&function, milis);

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
    // Ensure closure isnt dropped before the timeout is cleared
    // In the error paths it will be dropped naturally
    // i.e this is *not* to ensure its dropped, but to ensure it is not dropped early
    drop(closure);
}

/// Wait for the next browser animation frame.
pub async fn next_animation_frame() {
    let (tx, rx) = futures_channel::oneshot::channel();

    let (closure, function) = create_closure(tx);
    let res = crate::get_window().request_animation_frame(&function);
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
    // Ensure closure isnt dropped before the timeout is cleared
    // In the error paths it will be dropped naturally
    // i.e this is *not* to ensure its dropped, but to ensure it is not dropped early
    drop(closure);
}

/// Convert a `tx` to a `Function` that sends a message once called.
/// `Closure::once_into_js` is used to ensure that the closure is dropped after it is called.
fn create_closure(tx: futures_channel::oneshot::Sender<()>) -> (impl Any, Function) {
    let closure = Closure::once(move || {
        let _ = tx.send(());
    });

    let function = closure.as_ref().clone().into();

    (closure, function)
}

/// Wait for at least one mesassge on the channel
/// Returns `None` if the channel is closed
pub(crate) async fn drain_available<T>(
    rx: &mut UnboundedReceiver<T>,
) -> Option<impl Iterator<Item = T>> {
    let first = rx.next().await?;
    Some(DrainAvailableStream {
        rx,
        first: Some(first),
    })
}

/// A *iterator* that drains the available messages from a `UnboundedReceiver`.
struct DrainAvailableStream<'a, T> {
    /// The receiver to drain messages from.
    rx: &'a mut UnboundedReceiver<T>,
    /// The first message that was received.
    first: Option<T>,
}

impl<T> Iterator for DrainAvailableStream<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(first) = self.first.take() {
            return Some(first);
        }

        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        match self.rx.poll_next_unpin(&mut cx) {
            Poll::Ready(Some(item)) => Some(item),
            Poll::Ready(None) | Poll::Pending => None,
        }
    }
}
