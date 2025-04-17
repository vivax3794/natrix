//! Utility traits and structs
use futures_channel::mpsc::UnboundedReceiver;
use futures_core::stream::Stream;
use futures_util::stream::StreamExt;

/// A version of the stdlib `Any` with no type ids, downcasting, etc
/// Its the minimal possible dyn object, mainly used for keep alive.
pub(crate) trait SmallAny {}
impl<T> SmallAny for T {}

/// Panic in debug mode.
macro_rules! debug_expect {
    ($expr:expr, or($or:expr), $($msg:expr), *) => {
        {
            let res = $expr;
            match res {
                Some(value) => value,
                None => {
                    debug_assert!(false, $($msg),*);
                    $or
                }
            }
        }
    };
    ($expr:expr, $($msg:expr), *) => {
        let res = $expr;
        debug_assert!(res.is_ok(), $($msg),*);
    };
}
pub(crate) use debug_expect;

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
