//! Signals for tracking reactive dependencies and modifications.

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use crate::reactivity::component::Component;
use crate::reactivity::state::{HookKey, KeepAlive, State};

/// State passed to rendering callbacks
pub(crate) struct RenderingState<'s> {
    /// Push objects to this array to keep them alive as long as the parent context is valid.
    pub(crate) keep_alive: &'s mut Vec<KeepAlive>,
    /// The hooks that are a child of this
    pub(crate) hooks: &'s mut Vec<HookKey>,
    /// The parent render context, can be used to register it as a dependency of yourself
    pub(crate) parent_dep: HookKey,
}

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
    deps: Vec<HookKey>,
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
            deps: Vec::new(),
        }
    }

    /// Pop this signal's state and clear the read and written flags.
    ///
    /// This is `pub` only for use in macro generated `ComponentData` implementations.
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
    ///
    /// This is `pub` only for use in macro generated `ComponentData` implementations.
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
pub trait SignalMethods {
    /// Clear the `read` and `written` flags.
    fn clear(&mut self);
    /// Adds the given dependency to the hashset if the `read` flag is set.
    fn register_dep(&mut self, dep: HookKey);
    /// Return a mutable reference to the dependencies to facilitate efficient cleanup and
    /// deduplication in the `State struct`
    ///
    /// We are doing the cleaning in the `State` struct because it lets us deduplicate the changed
    /// hooks in `.update` without looping over the hashset twice.
    fn deps(&mut self) -> std::vec::Drain<'_, HookKey>;
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
            self.deps.push(dep);
        }
    }

    fn changed(&self) -> bool {
        self.written
    }

    fn deps(&mut self) -> std::vec::Drain<'_, HookKey> {
        self.deps.drain(..)
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

/// All reactive hooks will implement this trait to allow them to be stored as `dyn` objects.
pub(crate) trait ReactiveHook<C: Component> {
    /// Recalculate the hook and apply its update.
    ///
    /// Hooks should recall `ctx.reg_dep` with the you parameter to re-register any potential
    /// dependencies as the update method uses `.drain(..)` on dependencies (this is also to ensure
    /// reactive state that is only accessed in some conditions is recorded).
    fn update(&mut self, _ctx: &mut State<C>, _you: HookKey) -> UpdateResult;
    /// Return the list of hooks that should be dropped
    fn drop_us(self: Box<Self>) -> Vec<HookKey>;
}

/// The result of pre-update
pub(crate) enum UpdateResult {
    /// Do nothing extra
    Nothing,
    /// Drop the given hooks
    DropHooks(Vec<HookKey>),
    /// Run this hook after this one
    RunHook(HookKey),
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
