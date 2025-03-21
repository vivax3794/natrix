//! Utility traits and structs

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
