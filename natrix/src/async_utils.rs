//! Variosu async versions of js callback apis
use std::task::{Context, Poll};
use std::time::Duration;

use futures_channel::mpsc::UnboundedReceiver;
use futures_channel::oneshot;
use futures_util::stream::StreamExt;
use futures_util::task::noop_waker;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::Closure;
use web_sys::js_sys::Function;

use crate::utils::{DropGuard, debug_panic};

/// Sleeps for the given duration using js `setTimeout`.
///
/// Returns `None` if the given duration overflows a i32 milliseconds (~24 days).
/// Reasonably this is a rare condition to hit for sane code, hence you can use `sleep_milliseconds`
/// and pass in a i32 of milliseconds directly.
#[must_use]
pub fn sleep(duration: Duration) -> Option<impl Future<Output = ()>> {
    let milis = duration.as_millis().try_into().ok()?;
    Some(sleep_milliseconds(milis))
}

/// A helper function to wait for an asynchronous browser event with cancellation support.
///
/// This function encapsulates the common logic for setting up a oneshot channel,
/// creating a JS closure, calling a setup function (like `setTimeout` or `requestAnimationFrame`),
/// and managing cancellation with a `DropGuard`.
async fn wait_with_cancellation<T: Copy + 'static>(
    setup: impl FnOnce(&Function) -> Result<T, JsValue>,
    cancel: impl FnOnce(T) + 'static,
    setup_err_msg: &str,
    recv_err_msg: &str,
) {
    let (tx, rx) = oneshot::channel();
    let closure = Closure::once(move || {
        let _ = tx.send(());
    });
    let function: Function = closure.as_ref().clone().into();

    let Ok(id) = setup(&function) else {
        debug_panic!("{}", setup_err_msg);
        return;
    };

    let mut drop_guard = DropGuard::new(move || {
        cancel(id);
    });

    if rx.await.is_err() {
        debug_panic!("{}", recv_err_msg);
        return;
    }

    drop_guard.cancel();
}

/// Sleeps for the given milliseconds using js `setTimeout`.
///
/// # Why `i32`?
/// Because thats what the `web_sys` `setTimeout` binding uses.
pub async fn sleep_milliseconds(milis: i32) {
    wait_with_cancellation(
        |function| {
            crate::get_window()
                .set_timeout_with_callback_and_timeout_and_arguments_0(function, milis)
        },
        move |timeout_id| {
            crate::get_window().clear_timeout_with_handle(timeout_id);
        },
        "Failed to set timeout. This is a bug in the browser or the framework.",
        "Failed to receive timeout signal",
    )
    .await;
}

/// Wait for the next browser animation frame using `requestAnimationFrame`.
pub async fn next_animation_frame() {
    wait_with_cancellation(
        |function| crate::get_window().request_animation_frame(function),
        move |frame_id| {
            let res = crate::get_window().cancel_animation_frame(frame_id);
            if res.is_err() {
                debug_panic!(
                    "Failed to cancel animation frame. This is a bug in the browser or the framework."
                );
            }
        },
        "Failed to request animation frame. This is a bug in the browser or the framework.",
        "Failed to receive animation frame signal",
    )
    .await;
}

/// Wait for at least one mesassge on the channel
/// Returns `None` if the channel is closed
pub(crate) async fn recv_batch<T>(
    rx: &mut UnboundedReceiver<T>,
) -> Option<impl Iterator<Item = T>> {
    let first = rx.next().await?;
    Some(RecvBatchIter {
        rx,
        first: Some(first),
    })
}

/// A *iterator* that drains the available messages from a `UnboundedReceiver`.
struct RecvBatchIter<'a, T> {
    /// The receiver to drain messages from.
    rx: &'a mut UnboundedReceiver<T>,
    /// The first message that was received.
    first: Option<T>,
}

impl<T> Iterator for RecvBatchIter<'_, T> {
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
