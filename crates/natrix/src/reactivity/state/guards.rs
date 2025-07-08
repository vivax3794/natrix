//! Implementation of guards
#![cfg(false)] // TODO: Re-implment guards using Lens

pub use super::{Ctx, RenderCtx};

// MAYBE: Can we somehow abstract over immutable vs mutable getters in a way that lets a user write
// a provably pure getter that works for both?

/// Get a guard handle that can be used to retrieve the `Some` variant of a option without having to
/// use `.unwrap`.
/// Should be used to achieve find-grained reactivity (internally this uses `.watch` on `.is_some()`)
///
/// # Why?
/// The usecase can be seen by considering this logic:
/// ```rust
/// # use natrix::prelude::*;
/// # #[derive(State)]
/// # struct MyState {value: Option<u32>}
/// # impl State for MyState {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value) = *ctx.value {
///     e::div().text(value)
/// } else {
///     e::div().text("Is none")
/// }
/// # }}}
/// ```
/// The issue here is that the outer div (which might be a more expensive structure to create) is
/// recreated everytime `value` changes, even if it is `Some(0) -> Some(1)`
/// This is where you might reach for `ctx.watch`, and in fact that works perfectly:
/// ```rust
/// # use natrix::prelude::*;
/// # #[derive(State)]
/// # struct MyState {value: Option<u32>}
/// # impl State for MyState {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if ctx.watch(|ctx| ctx.value.is_some()) {
///     e::div().text(|ctx: R<Self>| ctx.value.unwrap())
/// } else {
///     e::div().text("Is none")
/// }
/// # }}}
/// ```
/// And this works, Now a change from `Some(0)` to `Some(1)` will only run the inner closure and
/// the outer div is reused. but there is one downside, we need `.unwrap` because the inner closure is
/// technically isolated, and this is ugly, and its easy to do by accident. and you might forget
/// the outer condition.
///
/// This is where guards come into play:
/// ```rust
/// # use natrix::prelude::*;
/// # use natrix::guard_option;
/// # #[derive(State)]
/// # struct MyState {value: Option<u32>}
/// # impl State for MyState {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value_guard) = guard_option!(|ctx| ctx.value.as_ref()) {
///     e::div().text(move |ctx: R<Self>| *ctx.get(&value_guard))
/// } else {
///     e::div().text("Is none")
/// }
/// # }}}
/// ```
/// Here `value_guard` is actually not the value at all, its a lightweight value thats can be
/// captured by child closures and basically is a way to say "I know that in this context this
/// value is `Some`"
///
/// Internally this uses `ctx.watch` and `.unwrap` (which should never fail)
///
/// ## Mutable returns
/// If you want to return a mutable reference to the value you can use the `@mut` version:
/// ```rust
/// # use natrix::prelude::*;
/// # use natrix::guard_option;
/// # #[derive(State)]
/// # struct MyState {value: Option<u32>}
/// # impl State for MyState {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value_guard) = guard_option!(@mut |ctx| ctx.value.as_mut()) {
///   e::button().on::<events::Click>(move |ctx: E<Self>, _, _| {
///     *ctx.get_mut(&value_guard) += 1;
///   }).generic()
/// } else {
///   e::div().text("Is none").generic()
/// }
/// # }}}
/// ```
///
/// You can also use a mutable guard in reactive closures via `get_downgrade`
/// ```rust
/// # use natrix::prelude::*;
/// # use natrix::guard_option;
/// # #[derive(State)]
/// # struct MyState {value: Option<u32>}
/// # impl State for MyState {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value_guard) = guard_option!(@mut |ctx| ctx.value.as_mut()) {
///   e::button()
///     .text(move |ctx: R<Self>| *ctx.get_downgrade(&value_guard))
///     .on::<events::Click>(move |ctx: E<Self>, _, _| {
///       *ctx.get_mut(&value_guard) += 1;
///     })
///     .generic()
/// } else {
///   e::div().text("Is none").generic()
/// }
/// # }}}
/// ```
///
/// **IMPORTANT**: Even tho the guard closure takes a mutable reference, you should not mutate it.
/// Instead it should be only be used to get a `&mut ...` to value you want.
///
/// ## Owned returns
/// By default this macro assumes the return value is `&T`, but if you want to return an owned
/// value you can use the `@owned` version:
/// ```rust
/// # use natrix::prelude::*;
/// # use natrix::guard_option;
/// # #[derive(State)]
/// # struct MyState {value: Option<u32>}
/// # impl State for MyState {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value_guard) = guard_option!(@owned |ctx| ctx.value) {
///    e::div().text(move |ctx: R<Self>| ctx.get_owned(&value_guard))
/// } else {
///    e::div().text("Is none")
/// }
/// # }}}
/// ```
#[macro_export]
macro_rules! guard_option {
    (| $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_some()) {
            Some($crate::macro_ref::Guard::new::<Self, _>(move |$ctx| {
                $expr.expect("Guard used on None value")
            }))
        } else {
            None
        }
    };
    (@mut | $ctx:ident | $expr:expr) => {
        if $ctx.watch_mut(move |$ctx| $expr.is_some()) {
            Some($crate::macro_ref::Guard::new_mut::<Self, _>(move |$ctx| {
                $expr.expect("Guard used on None value")
            }))
        } else {
            None
        }
    };
    (@owned | $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_some()) {
            Some($crate::macro_ref::Guard::new_owned::<Self, _>(
                move |$ctx| $expr.expect("Guard used on None value"),
            ))
        } else {
            None
        }
    };
}

/// Get a guard handle that can be used to retrieve the `Ok` variant of a option without having to
/// use `.unwrap`, or the `Err` variant.
#[macro_export]
macro_rules! guard_result {
    (| $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_ok()) {
            Ok($crate::macro_ref::Guard::new::<Self, _>(move |$ctx| {
                $expr.expect("Guard used on Err value")
            }))
        } else {
            Err($crate::macro_ref::Guard::new::<Self, _>(move |$ctx| {
                $expr.expect_err("Guard used on Ok value")
            }))
        }
    };
    (@mut | $ctx:ident | $expr:expr) => {
        if $ctx.watch_mut(move |$ctx| $expr.is_ok()) {
            Ok($crate::macro_ref::Guard::new_mut::<Self, _>(move |$ctx| {
                $expr.expect("Guard used on Err value")
            }))
        } else {
            Err($crate::macro_ref::Guard::new_mut::<Self, _>(move |$ctx| {
                $expr.expect_err("Guard used on Ok value")
            }))
        }
    };
    (@owned | $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_ok()) {
            Ok($crate::macro_ref::Guard::new_owned::<Self, _>(
                move |$ctx| $expr.expect("Guard used on Err value"),
            ))
        } else {
            Err($crate::macro_ref::Guard::new_owned::<Self, _>(
                move |$ctx| $expr.expect_err("Guard used on Ok value"),
            ))
        }
    };
}
/// This guard ensures that when it is in scope the data it was created for is `Some`
#[cfg_attr(feature = "nightly", must_not_suspend)]
#[derive(Clone, Copy)]
#[must_use]
pub struct Guard<F> {
    /// The closure for getting the value from a ctx
    getter: F,
}

impl<F> Guard<F> {
    #[doc(hidden)]
    #[inline]
    pub fn new<C, R>(getter: F) -> Self
    where
        F: for<'a> Fn(&'a Ctx<C>) -> &'a R,
        C: State,
    {
        Self { getter }
    }

    #[doc(hidden)]
    #[inline]
    pub fn new_mut<C, R>(getter: F) -> Self
    where
        F: for<'a> Fn(&'a mut Ctx<C>) -> &'a mut R,
        C: State,
    {
        Self { getter }
    }

    #[doc(hidden)]
    #[inline]
    pub fn new_owned<C, R>(getter: F) -> Self
    where
        F: Fn(&Ctx<C>) -> R,
        C: State,
    {
        Self { getter }
    }
}

impl<T: State> Ctx<T> {
    /// Get the unwrapped data referenced by this guard
    #[inline]
    pub fn get<'s, F, R>(&'s self, guard: &Guard<F>) -> &'s R
    where
        F: Fn(&'s Self) -> &'s R,
    {
        (guard.getter)(self)
    }

    /// Get the unwrapped data referenced by this guard, but owned
    #[inline]
    pub fn get_owned<F, R>(&self, guard: &Guard<F>) -> R
    where
        F: Fn(&Self) -> R,
    {
        (guard.getter)(self)
    }

    /// Get the unwrapped data referenced by this guard, but mut
    #[inline]
    pub fn get_mut<'s, F, R>(&'s mut self, guard: &Guard<F>) -> &'s mut R
    where
        F: Fn(&'s mut Self) -> &'s mut R,
    {
        (guard.getter)(self)
    }
}

impl<C: State> RenderCtx<'_, C> {
    /// Get a readonly reference from a mut guard
    #[inline]
    pub fn get_downgrade<F, R>(&mut self, guard: &Guard<F>) -> &R
    where
        F: Fn(&mut Ctx<C>) -> &mut R,
    {
        (guard.getter)(self.ctx)
    }
}
