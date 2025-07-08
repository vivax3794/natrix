//! Signals for tracking reactive dependencies and modifications.

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use indexmap::IndexSet;

use crate::error_handling::{do_performance_check, performance_lint};
use crate::prelude::State;
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
}

impl<T: 'static> State for Signal<T> {
    #[inline]
    fn clear(&mut self) {
        self.written = false;
        self.read.set(false);
    }

    fn reg_dep(&mut self, dep: HookKey) {
        if self.read.get() {
            self.deps.insert(dep);
        }

        if do_performance_check() {
            if self.deps.len() > 20 {
                performance_lint!("Signal dep list is {}", self.deps.len());
            }
        }
    }

    fn dirty_deps_lists(&mut self) -> impl Iterator<Item = indexmap::set::IntoIter<HookKey>> {
        let mut new = IndexSet::with_capacity(self.deps.len());
        std::mem::swap(&mut new, &mut self.deps);
        std::iter::once(new.into_iter())
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
    use super::Signal;
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

        assert!(foo.0.written);
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
