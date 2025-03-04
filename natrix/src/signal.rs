//! Signals for tracking reactive depdencies and modifications.

use std::cell::{Cell, RefCell};
use std::ops::{AddAssign, Deref, DerefMut, DivAssign, MulAssign, SubAssign};

use crate::state::{ComponentData, KeepAlive, State};
use crate::utils::{RcCmpPtr, WeakCmpPtr};

/// A `Rc` for a reactive hook
pub type RcDep<C> = RcCmpPtr<RefCell<Box<dyn ReactiveHook<C>>>>;

/// A `rc::Weak` for a reactive hook
pub type RcDepWeak<C> = WeakCmpPtr<RefCell<Box<dyn ReactiveHook<C>>>>;

/// State passed to rendering callbacks
pub(crate) struct RenderingState<'s> {
    /// Push objects to this array to keep them alive as long as the parent context is valid.
    pub(crate) keep_alive: &'s mut Vec<KeepAlive>,
}

/// A signal tracks reads and writes, as well as
pub struct Signal<T, C> {
    /// The data to be tracked.
    data: T,
    /// The flag for wether this signal has been written to
    written: bool,
    /// The flag for wether this signal has been read
    /// this is a `Cell` to allow for modification in `Deref`
    read: Cell<bool>,
    /// A hashset of the dependencies.
    ///
    /// Actually calling said depdencies is the responsibility of the `State` struct.
    /// Depdencies are also lazily removed by the `State` struct, and hence might contain stale
    /// pointers.
    deps: Vec<RcDepWeak<C>>,
}

impl<T, C> Signal<T, C> {
    /// Create a new signal with the specified data
    pub fn new(data: T) -> Self {
        Self {
            data,
            written: false,
            read: Cell::new(false),
            deps: Vec::new(),
        }
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
pub trait SignalMethods<C> {
    /// Clear the `read` and `written` flags.
    fn clear(&mut self);
    /// Adds the given depedency to the hashset if the `read` flag is set.
    fn register_dep(&mut self, dep: RcDepWeak<C>);
    /// Return a mutable reference to the depdencies to facilitate efficent cleanup and
    /// deduplication in the `State struct`
    ///
    /// We are doing the cleaning in the `State` struct because it lets us deduplicate the changed
    /// hooks in `.update` without looping over the hashset twice.
    fn deps(&mut self) -> &mut Vec<RcDepWeak<C>>;
    /// Return the value of the `written` field
    fn changed(&self) -> bool;
}

impl<T, C> SignalMethods<C> for Signal<T, C> {
    fn clear(&mut self) {
        self.written = false;
        self.read.set(false);
    }

    fn register_dep(&mut self, dep: RcDepWeak<C>) {
        if self.read.get() {
            self.deps.push(dep);
        }
    }

    fn changed(&self) -> bool {
        self.written
    }

    fn deps(&mut self) -> &mut Vec<RcDepWeak<C>> {
        &mut self.deps
    }
}

impl<T, C> Deref for Signal<T, C> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.read.set(true);
        &self.data
    }
}
impl<T, C> DerefMut for Signal<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.written = true;
        &mut self.data
    }
}

/// All reactive hooks will implement this trait to allow them to be stored as `dyn` objects.
pub(crate) trait ReactiveHook<C: ComponentData> {
    /// Recalculate the hook and apply its update.
    ///
    /// Hooks should recall `ctx.reg_dep` with the you paramater to re-register any potential
    /// depdencies.
    fn update(&mut self, ctx: &mut State<C>, you: &RcDepWeak<C>);
}

impl<T: PartialEq, C> PartialEq for Signal<T, C> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
impl<T: PartialEq, C> PartialEq<T> for Signal<T, C> {
    fn eq(&self, other: &T) -> bool {
        **self == *other
    }
}

impl<T: PartialOrd, C> PartialOrd for Signal<T, C> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }
}
impl<T: PartialOrd, C> PartialOrd<T> for Signal<T, C> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        (**self).partial_cmp(other)
    }
}

impl<R, T: AddAssign<R>, C> AddAssign<R> for Signal<T, C> {
    fn add_assign(&mut self, rhs: R) {
        **self += rhs;
    }
}
impl<R, T: SubAssign<R>, C> SubAssign<R> for Signal<T, C> {
    fn sub_assign(&mut self, rhs: R) {
        **self -= rhs;
    }
}
impl<R, T: MulAssign<R>, C> MulAssign<R> for Signal<T, C> {
    fn mul_assign(&mut self, rhs: R) {
        **self *= rhs;
    }
}
impl<R, T: DivAssign<R>, C> DivAssign<R> for Signal<T, C> {
    fn div_assign(&mut self, rhs: R) {
        **self /= rhs;
    }
}

