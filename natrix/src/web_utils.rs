//! Wrappers for js apis
#[cfg(feature = "async")]
use std::time::Duration;

pub use gloo;

/// Sleeps for the given duration using js `setTimeout`.
///
/// # Panics
/// If duration in miliseconds cant fit in a u32
#[cfg(feature = "async")]
pub async fn sleep(time: Duration) {
    let milis = u32::try_from(time.as_millis()).unwrap();
    gloo::timers::future::TimeoutFuture::new(milis).await;
}
