//! Implementation of core async features

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

pub use super::{EventToken, State};
use crate::error_handling::log_or_panic;
use crate::reactivity::component::Component;

impl<T: Component> State<T> {
    /// Get a wrapper around `Weak<RefCell<T>>` which provides a safer api that aligns with
    /// framework assumptions.
    pub fn deferred_borrow(&self, _token: EventToken) -> DeferredCtx<T> {
        DeferredCtx {
            inner: self.this.clone(),
        }
    }

    /// Spawn a async task in the local event loop, which will run on the next possible moment.
    pub fn use_async<C, F>(&self, token: EventToken, func: C)
    where
        C: FnOnce(DeferredCtx<T>) -> F,
        F: Future<Output = Option<()>> + 'static,
    {
        let deferred = self.deferred_borrow(token);
        let future = func(deferred);
        let future = async {
            let _ = future.await;
        };

        let future = PanicCheckFuture { inner: future };

        wasm_bindgen_futures::spawn_local(future);
    }
}

use std::cell::RefMut;
use std::marker::PhantomData;

/// A wrapper future that checks `has_panicked` before resolving.
///
/// If you are using `wasm_bindgen_futures` directly you should wrap your futures in this.
#[pin_project::pin_project]
pub struct PanicCheckFuture<F> {
    /// The future to run
    #[pin]
    pub inner: F,
}

impl<F: Future> Future for PanicCheckFuture<F> {
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if crate::panics::has_panicked() {
            std::task::Poll::Pending
        } else {
            self.project().inner.poll(cx)
        }
    }
}

/// A combiend `Weak` and `RefCell` that facilities upgrading and borrowing as a shared
/// operation
#[must_use]
pub struct DeferredCtx<T: Component> {
    /// The `Weak<RefCell<T>>` in question
    inner: Weak<RefCell<State<T>>>,
}

// We put a bound on `'p` so that users are not able to store the upgraded reference (unless
// they want to use ouroboros themself to store it alongside the weak).
#[ouroboros::self_referencing]
struct DeferredRefInner<'p, T: Component> {
    rc: Rc<RefCell<State<T>>>,
    lifetime: PhantomData<&'p ()>,
    #[borrows(rc)]
    #[covariant]
    reference: RefMut<'this, State<T>>,
}

/// a `RefMut` that also holds a `Rc`.
/// See the `DeferredCtx::borrow_mut` on drop semantics and safety
#[cfg_attr(feature = "nightly", must_not_suspend)]
#[must_use]
pub struct DeferredRef<'p, T: Component>(DeferredRefInner<'p, T>);

impl<T: Component> DeferredCtx<T> {
    /// Borrow this `Weak<RefCell<...>>`, this will create a `Rc` for as long as the borrow is
    /// active. Returns `None` if the component was dropped. Its recommended to use the
    /// following construct to safely cancel async tasks:
    /// ```ignore
    /// let Some(mut borrow) = ctx.borrow_mut() else {return;};
    /// // ...
    /// drop(borrow);
    /// foo().await;
    /// let Some(mut borrow) = ctx.borrow_mut() else {return;};
    /// // ...
    /// ```
    ///
    /// # Reactivity
    /// Calling this function clears the internal reactive flags (which is safe as long as the
    /// borrow safety rules below are followed).
    /// Once this value is dropped it will trigger a reactive update for any changed fields.
    ///
    /// # Borrow Safety
    /// The framework guarantees that it will never hold a borrow between event calls.
    /// This means the only source of panics is if you are holding a borrow when you yield to
    /// the event loop, i.e you should *NOT* hold this value across `.await` points.
    /// framework will regularly borrow the state on any registered event handler trigger, for
    /// example a user clicking a button.
    ///
    /// Keeping this type across an `.await` point or otherwise yielding control to the event
    /// loop while the borrow is active could also lead to reactivity failrues and desyncs.
    ///
    /// ## Nightly
    /// The nightly feature flag enables a lint to detect this misuse.
    /// See the [Features]() chapther for details on how to set it up (it requires a bit more
    /// setup than just turning on the feature flag).
    #[must_use]
    pub fn borrow_mut(&self) -> Option<DeferredRef<'_, T>> {
        let rc = self.inner.upgrade()?;
        let borrow = DeferredRefInner::try_new(rc, PhantomData, |rc| rc.try_borrow_mut());

        let Ok(mut borrow) = borrow else {
            log_or_panic!(
                "Deferred state borrowed while already borrowed. This might happen due to holding it across a yield point"
            );
            return None;
        };

        borrow.with_reference_mut(|ctx| ctx.clear());
        Some(DeferredRef(borrow))
    }
}

impl<T: Component> Deref for DeferredRef<'_, T> {
    type Target = State<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.borrow_reference()
    }
}
impl<T: Component> DerefMut for DeferredRef<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.with_reference_mut(|cell| &mut **cell)
    }
}

impl<T: Component> Drop for DeferredRef<'_, T> {
    fn drop(&mut self) {
        self.0.with_reference_mut(|ctx| {
            ctx.update();
        });
    }
}
