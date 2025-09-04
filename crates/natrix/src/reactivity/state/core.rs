//! State core struct and constructors

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use crate::error_handling::log_or_panic;
use crate::reactivity::State;
use crate::reactivity::render_callbacks::RenderingState;
use crate::reactivity::state::hook_manager::HookStore;

/// The core framework state, also holds user data.
pub(crate) struct InnerCtx<T: State> {
    /// The user (macro) defined reactive struct
    pub(crate) data: T,
    /// A weak reference to ourself, so that event handlers can easially get a weak reference
    /// without having to pass it around in every api
    pub(crate) this: Weak<RefCell<Self>>,
    /// Reactive hooks
    pub(crate) hooks: HookStore<T>,
}

impl<T: State> InnerCtx<T> {
    /// Create a minimal instance of this without wrapping in Self
    ///
    /// Warning the `Weak` reference is not set up yet
    pub(crate) fn create_base(data: T) -> Self {
        Self {
            data,
            this: Weak::new(),
            hooks: HookStore::new(),
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
pub struct RenderCtx<'c, 's, C: State> {
    /// The inner context
    pub(crate) ctx: &'c mut InnerCtx<C>,
    /// The render state for this state
    pub(crate) render_state: RenderingState<'s>,
}

impl<C: State> Deref for RenderCtx<'_, '_, C> {
    type Target = C;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.ctx.data
    }
}

/// A event ctx.
pub struct EventCtx<'c, C: State>(pub(crate) &'c mut InnerCtx<C>);

impl<C: State> Deref for EventCtx<'_, C> {
    type Target = C;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0.data
    }
}
impl<C: State> DerefMut for EventCtx<'_, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.data
    }
}
