//! Implementation of core async features

use std::cell::RefCell;
use std::rc::Weak;

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

impl<T: Component> DeferredCtx<T> {
    /// Run a function on the component state, returning `None` if the component was dropped.
    ///
    /// # Reactivity
    /// Calling this function clears the internal reactive flags.
    /// Once this value is dropped it will trigger a reactive update for any changed fields.
    #[must_use]
    pub fn update<R>(&self, func: impl FnOnce(&mut State<T>) -> R) -> Option<R> {
        let rc = self.inner.upgrade()?;
        let Ok(mut borrow) = rc.try_borrow_mut() else {
            log_or_panic!(
                "Deferred state borrowed while already borrowed. This might happen due to holding it across a yield point"
            );
            return None;
        };

        let result = borrow.track_changes(func);
        Some(result)
    }
}
