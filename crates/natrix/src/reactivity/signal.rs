//! Signals for tracking reactive dependencies and modifications.

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use indexmap::IndexSet;

use crate::error_handling::{do_performance_check, performance_lint};
use crate::prelude::State;
use crate::reactivity::state::{HookDepListIter, HookKey};

/// A signal tracks reads and writes to a value, as well as dependencies.
pub struct Signal<T> {
    /// The data to be tracked.
    data: T,
    /// The flag for whether this signal has been read or written to.
    /// this is a `Cell` to allow for modification in `Deref`
    touched: Cell<bool>,
    /// A collection of the dependencies.
    deps: IndexSet<HookKey>,
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
            touched: Cell::new(false),
            deps: IndexSet::new(),
        }
    }
}

impl<T: 'static> State for Signal<T> {
    type SignalState = bool;

    fn reg_dep(&mut self, dep: HookKey) {
        if self.touched.take() {
            self.deps.insert(dep);
        }

        if do_performance_check() {
            if self.deps.len() > 20 {
                performance_lint!("Signal dep list is {}", self.deps.len());
            }
        }
    }

    fn dirty_deps_lists(&mut self, collector: &mut Vec<HookDepListIter>) {
        if self.touched.take() {
            let mut new = IndexSet::with_capacity(self.deps.len());
            std::mem::swap(&mut new, &mut self.deps);
            collector.push(new.into_iter());
        }
    }

    fn pop_state(&mut self) -> Self::SignalState {
        self.touched.take()
    }
    fn set_state(&mut self, dirty: Self::SignalState) {
        self.touched.set(dirty);
    }
}

impl<T> Deref for Signal<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.touched.set(true);
        &self.data
    }
}
impl<T> DerefMut for Signal<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.touched.set(true);
        &mut self.data
    }
}

impl<T: Default> Default for Signal<T> {
    fn default() -> Self {
        Self::new(T::default())
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
        assert!(foo.0.touched.get());
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

        assert!(foo.0.touched.get());
    }
}
