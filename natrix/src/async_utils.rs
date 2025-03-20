//! Async utils
use std::time::Duration;

pub use futures;
pub use futures::{join, select, try_join};

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
