//! Utility traits and structs
use futures_channel::mpsc::UnboundedReceiver;
use futures_core::stream::Stream;
use futures_util::stream::StreamExt;

/// A version of the stdlib `Any` with no type ids, downcasting, etc
/// Its the minimal possible dyn object, mainly used for keep alive.
pub(crate) trait SmallAny {}
impl<T> SmallAny for T {}

#[cfg(nightly)]
pub(crate) use std::hint::cold_path;

/// Stable version of `cold_path`
/// Likely does less optimization I think?
/// But better than nothing.
#[cfg(not(nightly))]
#[cold]
pub(crate) fn cold_path() {}

/// Panic in debug mode.
macro_rules! debug_expect {
    ($expr:expr, or($or:expr), $($msg:expr), *) => {
        {
            let res = $expr;
            match res {
                Some(value) => value,
                None => {
                    crate::utils::debug_panic!($($msg),*);
                    $or
                }
            }
        }
    };
    ($expr:expr, $($msg:expr), *) => {
        let res = $expr;
        match res {
            Ok(_) => {},
            Err(_) => {
                crate::utils::cold_path();
            }
        }
    };
}

/// Panic on debug builds only
macro_rules! debug_panic {
    ($($msg:expr),*) => {
        crate::utils::cold_path();
        if cfg!(debug_assertions) {
            panic!($($msg),*);
        }
    };
}

pub(crate) use {debug_expect, debug_panic};

/// Wait for at least one mesassge on the channel
/// And then return all of the available messages
/// Returns `None` if the channel is closed
pub(crate) async fn recv_all<T>(rx: &mut UnboundedReceiver<T>) -> Option<Vec<T>> {
    let first = rx.next().await?;
    let mut messages = Vec::new();
    // The lower bound of the size hint is the current amount of messages
    // And we know there wont be more since this is a single threaded runtime
    messages.reserve_exact(rx.size_hint().0.saturating_add(1));
    messages.push(first);

    // This returns `Err` when there are no more messages
    // While the docs say to not use this from a Future,
    // It also says its okay if we otherwise will be notified when it is no longer empty.
    // *Technically speaking* it is using it wrong to hit the `Err` case.
    // But it is the only way to get all messages (afaik).
    while let Ok(Some(message)) = rx.try_next() {
        messages.push(message);
    }

    Some(messages)
}
