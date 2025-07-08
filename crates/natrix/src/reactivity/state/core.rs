//! State core struct and constructors

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use crate::error_handling::log_or_panic;
use crate::reactivity::State;
use crate::reactivity::render_callbacks::RenderingState;
use crate::reactivity::state::hook_manager::HookStore;

/// The core component state, stores all framework data
pub struct Ctx<T: State> {
    /// The user (macro) defined reactive struct
    pub(crate) data: T,
    /// A weak reference to ourself, so that event handlers can easially get a weak reference
    /// without having to pass it around in every api
    pub(crate) this: Weak<RefCell<Self>>,
    /// Reactive hooks
    pub(crate) hooks: HookStore<T>,
    /// Messages gotten while we were borrowed
    pub(super) deferred_messages: Rc<super::messages::DeferredMessageQueue>,
}

impl<T: State> Deref for Ctx<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: State> DerefMut for Ctx<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: State> Ctx<T> {
    /// Create a minimal instance of this without wrapping in Self
    ///
    /// Warning the `Weak` reference is not set up yet
    pub(crate) fn create_base(data: T) -> Self {
        Self {
            data,
            this: Weak::new(),
            hooks: HookStore::new(),
            deferred_messages: Rc::default(),
        }
    }

    /// Convert this into a finlized state by populating `Weak` and returning a Rc
    pub(crate) fn finalize(self) -> Rc<RefCell<Self>> {
        let this = Rc::new(RefCell::new(self));

        if let Ok(mut borrow) = this.try_borrow_mut() {
            borrow.this = Rc::downgrade(&this);
        } else {
            log_or_panic!("State (somehow) already borrowed in `finalize");
        }

        this
    }

    /// Create a new instance of the state, returning a `Rc` to it
    pub(crate) fn new(data: T) -> Rc<RefCell<Self>> {
        Self::create_base(data).finalize()
    }
}

/// Wrapper around a mutable state that only allows read-only access
///
/// This holds a mutable state to facilitate a few rendering features such as `.watch`
pub struct RenderCtx<'c, C: State> {
    /// The inner context
    pub(crate) ctx: &'c mut Ctx<C>,
    /// The render state for this state
    pub(crate) render_state: RenderingState<'c>,
}

impl<C: State> Deref for RenderCtx<'_, C> {
    type Target = Ctx<C>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}
