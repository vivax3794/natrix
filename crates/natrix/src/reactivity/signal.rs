//! Signals for tracking reactive dependencies and modifications.

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use indexmap::IndexSet;

use crate::error_handling::{do_performance_check, performance_lint};
use crate::reactivity::state::HookKey;

// TODO: Make the transformation of component structs into per-field signals a more generic
// process, and allow nesting signals in a smart way. for example you can annotate your `Book`
// struct with some macro, and when you use `Book` as a field of your component you actually get
// reactivity on the level of the books fields and not the whole book struct.

/// A signal tracks reads and writes to a value, as well as dependencies.
pub struct Signal<T> {
    /// The data to be tracked.
    data: T,
    /// The flag for whether this signal has been written to
    written: bool,
    /// The flag for whether this signal has been read
    /// this is a `Cell` to allow for modification in `Deref`
    read: Cell<bool>,
    /// A vector of the dependencies.
    ///
    /// Actually calling said dependencies is the responsibility of the `State` struct.
    /// Dependencies are also lazily removed by the `State` struct, and hence might contain stale
    /// pointers.
    deps: IndexSet<HookKey>,
}

/// The `written` and `read` flags of a signal extracted
#[derive(Copy, Clone)]
pub struct SignalState {
    /// Was the signal written to
    written: bool,
    /// Was the signal read
    read: bool,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T> Signal<T> {
    /// Create a new signal with the specified data
    pub fn new(data: T) -> Self {
        Self {
            data,
            written: false,
            read: Cell::new(false),
            deps: IndexSet::new(),
        }
    }

    /// Pop this signal's state and clear the read and written flags.
    #[doc(hidden)]
    pub fn pop_state(&mut self) -> SignalState {
        let result = SignalState {
            written: self.written,
            read: self.read.get(),
        };
        self.clear();
        result
    }

    /// Set this signal's state..
    #[doc(hidden)]
    pub fn set_state(&mut self, state: SignalState) {
        self.written = state.written;
        self.read.set(state.read);
    }
}

/// Methods for signals that arent generic over the contained data.
///
/// The use case of this trait is allowing the `State` struct
/// to work on a `Vec<&dyn SignalMethods<C>>`, which means the derive macro does not need to
/// generate a lot of a delegation methods.
///
/// Performance: This approach leads to cleaner-- less macro heavy --code, but does have a
/// performance hit via vtable and vector allocation overhead.
#[doc(hidden)]
pub trait SignalMethods {
    /// Clear the `read` and `written` flags.
    fn clear(&mut self);
    /// Adds the given dependency to the hashset if the `read` flag is set.
    fn register_dep(&mut self, dep: HookKey);
    /// Clear out the hooks and return a iterator over them.
    ///
    /// We are doing the cleaning in the `State` struct because it lets us deduplicate the changed
    /// hooks in `.update` without looping over the vec twice.
    ///
    /// (This is a concrete type instead of `impl Iterator...` because this trait needs to be
    /// object safe.)
    fn drain_dependencies(&mut self) -> Vec<HookKey>;
    /// Return the value of the `written` field
    fn changed(&self) -> bool;
}

impl<T> SignalMethods for Signal<T> {
    fn clear(&mut self) {
        self.written = false;
        self.read.set(false);
    }

    fn register_dep(&mut self, dep: HookKey) {
        if self.read.get() {
            self.deps.insert(dep);
            if do_performance_check() {
                if self.deps.len() > 20 {
                    performance_lint!(
                        "`{}` signal has {} dependencies",
                        std::any::type_name::<T>(),
                        self.deps.len()
                    );
                }
            }
        }
    }

    fn changed(&self) -> bool {
        self.written
    }

    fn drain_dependencies(&mut self) -> Vec<HookKey> {
        self.deps.drain(..).collect()
    }
}

impl<T> Deref for Signal<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.read.set(true);
        &self.data
    }
}
impl<T> DerefMut for Signal<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.written = true;
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::{Signal, SignalMethods};
    // We put signals in a struct to simulate the real usage pattern where they are always fields
    // in a &ref
    struct Holder<T>(Signal<T>);

    #[test]
    fn reading() {
        let foo = &Holder(Signal::new(10));
        assert_eq!(*foo.0, 10);
        assert!(foo.0.read.get());
    }

    #[test]
    fn modify() {
        let foo = &mut Holder(Signal::new(10));
        *foo.0 = 20;

        assert!(foo.0.changed());
        assert_eq!(*foo.0, 20);
    }

    #[test]
    fn debug() {
        let data = "Hello World";
        let foo = &Holder(Signal::new(data));

        assert_eq!(format!("{:?}", foo.0), format!("{data:?}"));

        assert!(foo.0.read.get());
    }
}
