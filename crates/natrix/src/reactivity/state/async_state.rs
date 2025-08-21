//! Implementation of core async features

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Weak;

use super::InnerCtx;
use crate::EventCtx;
use crate::error_handling::log_or_panic;
use crate::reactivity::State;

impl<T: State> EventCtx<'_, T> {
    /// Spawn a async task in the local event loop, which will run on the next possible moment.
    pub fn use_async<C, F>(&self, func: C)
    where
        C: FnOnce(AsyncCtxHandle<T>) -> F,
        F: Future<Output = Option<()>> + 'static,
    {
        let handle = AsyncCtxHandle {
            inner: self.0.this.clone(),
        };
        let future = func(handle);
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

/// A ctx for async context.
pub struct AsyncCtx<'s, S: State>(pub(crate) &'s mut InnerCtx<S>);
impl<S: State> Deref for AsyncCtx<'_, S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &self.0.data
    }
}
impl<S: State> DerefMut for AsyncCtx<'_, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.data
    }
}

/// A combiend `Weak` and `RefCell` that facilities upgrading and borrowing as a shared
/// operation and ensures you cant cause borrow errors.
#[must_use]
pub struct AsyncCtxHandle<T: State> {
    /// The `Weak<RefCell<T>>` in question
    inner: Weak<RefCell<InnerCtx<T>>>,
}

impl<T: State> Clone for AsyncCtxHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: State> AsyncCtxHandle<T> {
    /// Run a function on the component state, returning `None` if the component was dropped.
    ///
    /// # Reactivity
    /// Calling this function clears the internal reactive flags.
    /// And causes a update to the UI when the closure exists.
    #[must_use]
    pub fn update<R>(&self, func: impl FnOnce(AsyncCtx<T>) -> R) -> Option<R> {
        let rc = self.inner.upgrade()?;
        let Ok(mut borrow) = rc.try_borrow_mut() else {
            log_or_panic!("State borrowed while already borrowed.");
            return None;
        };

        let result = borrow.track_changes(|ctx| func(AsyncCtx(ctx)));
        Some(result)
    }
}
