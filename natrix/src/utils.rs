//! Utility traits and structs

/// Cold path hint, causes compiler to better optimize unlikely error paths.
#[cold]
pub(crate) fn cold_path() {}

/// Panic on `Err` value in debug mode.
///
/// See `debug_panic` for usage philosophy.
macro_rules! debug_expect {
    ($expr:expr, $($msg:expr),*) => {
        let res = $expr;
        match res {
            Ok(_) => {}
            Err(_) => {
                $crate::utils::debug_panic!($($msg),*);
            }
        }
    };
}

/// Panic on debug builds only
///
/// This lives in a gray zone between normal panics and Result/Option.
/// It should be preferred over `panic!` in basically every case, but whether its better than
/// `Result` or `Option` comes down to what a caller can reasonably be expected to do in the error
/// case.
///
/// For example, most uses of this is in framework internals, where the entire callstack is pure
/// natrix code. there is no library user to make a decision. so if we used `Result`/`Option` we
/// would just push the "ignore it" decision to the top of the stack.
/// instead panicking early gives a better pointer to the error location, as well as always for
/// easier recovery paths in release builds.
///
/// This should only be used when the error path is the result of a bug in the framework or user
/// code, *not for errors that can reasonably happen because of end user input*. For example if say
/// `replaceChild` call fails thats very unexpected and we flag it in debug builds, because likely
/// the developer did some strange dom modification outside of the framework.
/// But in release, well we can survive missing a dom update.
///
/// TL;DR: Use this when an error both indicates a serious bug in framework or user code, and there
/// is a reasonable way to recover from it in release builds.
macro_rules! debug_panic {
    ($($msg:expr),*) => {
        $crate::utils::cold_path();


        ::log::error!($($msg),*);
        if cfg!(debug_assertions) {
            panic!($($msg),*);

        }
    };
}

/// Log a error to the console
pub(crate) use {debug_expect, debug_panic};

/// A guard that executes a callback when dropped.
///
/// This is primarily used to cleanup js resources when stuff like a Future is dropped.
pub(crate) struct DropGuard<F>
where
    F: FnOnce(),
{
    /// The callback to be executed on drop.
    callback: Option<F>,
}

impl<F> DropGuard<F>
where
    F: FnOnce(),
{
    /// Creates a new guard that will call the provided function on drop.
    pub(crate) fn new(callback: F) -> Self {
        Self {
            callback: Some(callback),
        }
    }

    /// Disables the callback, preventing it from being called on drop.
    pub(crate) fn cancel(&mut self) {
        self.callback = None;
    }
}

impl<F> Drop for DropGuard<F>
where
    F: FnOnce(),
{
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            callback();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn test_drop_guard_basic_functionality() {
        let called = Cell::new(false);

        {
            let _guard = DropGuard::new(|| called.set(true));
            assert!(!called.get()); // Not called yet
        } // guard drops here

        assert!(called.get()); // Called after drop
    }

    #[test]
    fn test_drop_guard_cancel() {
        let called = Cell::new(false);

        {
            let mut guard = DropGuard::new(|| called.set(true));
            guard.cancel();
        } // guard drops here, but callback was canceled

        assert!(!called.get());
    }

    #[test]
    fn test_multiple_drop_guards() {
        let counter = Cell::new(0);

        {
            let _guard1 = DropGuard::new(|| counter.set(counter.get() + 1));
            let _guard2 = DropGuard::new(|| counter.set(counter.get() + 2));
            let _guard3 = DropGuard::new(|| counter.set(counter.get() + 3));

            assert_eq!(counter.get(), 0);
        } // Guards drop in reverse order (LIFO)

        assert_eq!(counter.get(), 6); // 3 + 2 + 1
    }

    #[test]
    #[should_panic(expected = "Callback panic")]
    #[expect(clippy::panic, reason = "Its a test")]
    fn test_drop_guard_panicking_callback() {
        {
            let _guard = DropGuard::new(|| panic!("Callback panic"));
        } // guard drops here and should panic
    }

    #[test]
    fn test_drop_guard_with_captured_values() {
        let mut value = String::from("initial");

        {
            let _guard = DropGuard::new(|| {
                value = String::from("modified");
            });
        }

        assert_eq!(value, "modified");
    }

    #[test]
    fn test_nested_drop_guards() {
        let counter = Cell::new(0);

        {
            let _outer = DropGuard::new(|| {
                counter.set(counter.get() + 1);

                let _inner = DropGuard::new(|| {
                    counter.set(counter.get() + 10);
                });
                // inner guard drops here (inside outer callback)
            });
            // outer drops here
        }

        assert_eq!(counter.get(), 11); // 1 + 10
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic(expected = "Error in release mode"))]
    fn test_debug_expect() {
        debug_expect!(Err::<(), _>("error"), "Error in release mode");
    }

    #[test]
    #[cfg_attr(
        debug_assertions,
        should_panic(expected = "This won't panic in release")
    )]
    fn test_debug_panic() {
        // Should not panic in release mode
        debug_panic!("This won't panic in release");
    }
}
