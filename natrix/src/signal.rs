//! Signals for tracking reactive depdencies and modifications.

use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};

use crate::state::{ComponentData, KeepAlive, State};
use crate::utils::{HashSet, RcCmpPtr, WeakCmpPtr};

/// A `Rc` for a reactive hook
pub type RcDep<C> = RcCmpPtr<RefCell<Box<dyn ReactiveHook<C>>>>;

/// A `rc::Weak` for a reactive hook
pub type RcDepWeak<C> = WeakCmpPtr<RefCell<Box<dyn ReactiveHook<C>>>>;

/// State passed to rendering callbacks
pub(crate) struct RenderingState<'s> {
    /// Push objects to this array to keep them alive as long as the parent context is valid.
    pub(crate) keep_alive: &'s mut Vec<KeepAlive>,
}

pub struct Signal<T, C> {
    data: T,
    written: bool,
    read: Cell<bool>,
    deps: HashSet<RcDepWeak<C>>,
}

impl<T, C> Signal<T, C> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            written: false,
            read: Cell::new(false),
            deps: HashSet::default(),
        }
    }
}

pub trait SignalMethods<C> {
    fn clear(&mut self);
    fn register_dep(&mut self, dep: RcDepWeak<C>);
    fn deps(&mut self) -> &mut HashSet<RcDepWeak<C>>;
    fn changed(&self) -> bool;
}

impl<T, C> SignalMethods<C> for Signal<T, C> {
    #[inline(always)]
    fn clear(&mut self) {
        self.written = false;
        self.read.set(false);
    }

    #[inline(always)]
    fn register_dep(&mut self, dep: RcDepWeak<C>) {
        if self.read.get() {
            self.deps.insert(dep);
        }
    }

    #[inline(always)]
    fn changed(&self) -> bool {
        self.written
    }

    #[inline(always)]
    fn deps(&mut self) -> &mut HashSet<RcDepWeak<C>> {
        &mut self.deps
    }
}

impl<T, C> Deref for Signal<T, C> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.read.set(true);
        &self.data
    }
}
impl<T, C> DerefMut for Signal<T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.written = true;
        &mut self.data
    }
}

pub(crate) trait ReactiveHook<C: ComponentData> {
    fn update(&mut self, ctx: &mut State<C>, you: RcDepWeak<C>);
}
