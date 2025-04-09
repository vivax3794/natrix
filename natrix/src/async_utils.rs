//! Async utils
#![cfg(feature = "async_utils")]
use std::time::Duration;

pub use futures;
pub use futures::{join, select, try_join};

use crate::utils::debug_expect;

/// Sleeps for the given duration using js `setTimeout`.
pub async fn sleep(time: Duration) {
    let milis = if let Ok(milis) = u32::try_from(time.as_millis()) {
        milis
    } else {
        debug_assert!(
            false,
            "Sleep duration {}ms overflows `u32` (will use `u32::MAX` in release mode.)",
            time.as_millis()
        );
        u32::MAX
    };

    gloo::timers::future::TimeoutFuture::new(milis).await;
}

/// Wait for the next browser animation frame.
pub async fn next_animation_frame() {
    let (tx, rx) = futures::channel::oneshot::channel();

    let handle = gloo::render::request_animation_frame(move |_| {
        let _ = tx.send(());
    });

    debug_expect!(rx.await, "Failed to receive animation frame signal");

    drop(handle);
}
