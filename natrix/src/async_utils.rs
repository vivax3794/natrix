//! Async utils
use std::time::Duration;

pub use futures;
pub use futures::{join, select, try_join};

/// Sleeps for the given duration using js `setTimeout`.
///
/// # Panics
/// If duration in miliseconds cant fit in a u32
pub async fn sleep(time: Duration) {
    let milis = u32::try_from(time.as_millis())
        .expect("Sleep duration overflows u32::MAX (in miliseconds)");
    gloo::timers::future::TimeoutFuture::new(milis).await;
}
