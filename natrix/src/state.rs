//! Types for handling the component state

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use crate::component::ComponentBase;
use crate::signal::{RcDepWeak, SignalMethods};
use crate::utils::{HashSet, SmallAny};

/// Trait implemented on the reactive struct generated by the derive macro
pub trait ComponentData: Sized + 'static {
    /// The type of the returned signal fields.
    /// This should be a [...; N]
    type FieldRef<'a>: IntoIterator<Item = &'a mut dyn SignalMethods<Self>>;

    /// Returns mutable references to the signals
    #[doc(hidden)]
    fn signals_mut(&mut self) -> Self::FieldRef<'_>;
}

/// Alias for `Box<dyn SmallAny>`
/// for keeping specific objects alive in memory such as `Closure` and `Rc`
pub(crate) type KeepAlive = Box<dyn SmallAny>;

/// The core component state, stores all framework data
pub struct State<T> {
    /// The user (macro) defined reactive struct
    pub(crate) data: T,
    /// A weak reference to ourself, so that event handlers can easially get a weak reference
    /// without having to pass it around in every api
    this: Option<Weak<RefCell<Self>>>,
}

impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// A type alias for `State<C::Data>`, should be prefered in closure argument hints.
/// such as `|ctx: &S<Self>| ...`
pub type S<C> = State<<C as ComponentBase>::Data>;

impl<T> State<T> {
    /// Create a new instance of the state, returning a `Rc` to it
    pub(crate) fn new(data: T) -> Rc<RefCell<Self>> {
        let this = Self { data, this: None };
        let this = Rc::new(RefCell::new(this));

        this.borrow_mut().this = Some(Rc::downgrade(&this));

        this
    }

    /// Get a weak reference to this state
    #[inline]
    pub(crate) fn weak(&self) -> Weak<RefCell<Self>> {
        self.this.as_ref().expect("Weak not set").clone()
    }
}

impl<T: ComponentData> State<T> {
    /// Clear all signals
    pub(crate) fn clear(&mut self) {
        for signal in self.data.signals_mut() {
            signal.clear();
        }
    }

    /// Register a dependency for all read signals
    pub(crate) fn reg_dep(&mut self, dep: &RcDepWeak<T>) {
        for signal in self.data.signals_mut() {
            signal.register_dep(dep.clone());
        }
    }

    /// Loop over signals and update any depdant hooks for changed signals
    pub(crate) fn update(&mut self) {
        #[allow(clippy::mutable_key_type)]
        let mut hooks = HashSet::default();
        for signal in self.data.signals_mut() {
            if signal.changed() {
                for hook in signal.deps().drain(..) {
                    hooks.insert(hook);
                }
            }
        }

        for hook in hooks {
            if let Some(hook_strong) = hook.0.upgrade() {
                hook_strong.borrow_mut().update(self, &hook);
            }
        }
    }
}
