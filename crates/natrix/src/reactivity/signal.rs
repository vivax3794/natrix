//! Signals for tracking reactive dependencies and modifications.

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use indexmap::IndexSet;

use crate::error_handling::{do_performance_check, log_or_panic, performance_lint};
use crate::prelude::State;
use crate::reactivity::state::HookKey;
use crate::reactivity::statics;

/// A signal tracks reads and writes to a value, as well as dependencies.
pub struct Signal<T> {
    /// The data to be tracked.
    data: T,
    /// A collection of the dependencies.
    deps: RefCell<IndexSet<HookKey>>,
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
            deps: RefCell::new(IndexSet::new()),
        }
    }
}

impl<T: 'static> State for Signal<T> {}

impl<T> Deref for Signal<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        if let Some(hook) = statics::current_hook() {
            if let Ok(mut deps) = self.deps.try_borrow_mut() {
                deps.insert(hook);
                if do_performance_check() {
                    if deps.len() > 20 {
                        performance_lint!("Signal deps list over 20");
                    }
                }
            } else {
                log_or_panic!("Signal deps list already borrowed");
            }
        }

        &self.data
    }
}
impl<T> DerefMut for Signal<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let deps = self.deps.get_mut();
        if let Some(hook) = statics::current_hook() {
            deps.insert(hook);

            if do_performance_check() {
                if deps.len() > 20 {
                    performance_lint!("Signal deps list over 20");
                }
            }
        } else if !deps.is_empty() {
            statics::reg_dirty_list(|| {
                let mut new = IndexSet::with_capacity(deps.len());
                std::mem::swap(&mut new, deps);
                new.into_iter()
            });
        }

        &mut self.data
    }
}

impl<T: Default> Default for Signal<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
