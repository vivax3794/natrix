//! Implementation of guards

use std::marker::PhantomData;

use super::{RenderCtx, State};
use crate::lens::{self, Lens, LensInner};

impl<C: State> RenderCtx<'_, '_, C> {
    /// Get a guard lens that can be used to retrieve the `Some` variant of a option without having to
    /// use `.unwrap`.
    /// Should be used to achieve find-grained reactivity (internally this uses `.watch` on `.is_some()`)
    ///
    /// # Why?
    /// The usecase can be seen by considering this logic:
    /// ```rust
    /// # use natrix::prelude::*;
    /// # #[derive(State)]
    /// # struct App {value: Signal<Option<u32>>}
    /// # fn render() -> impl Element<App> {
    /// # |ctx: RenderCtx<App>| {
    /// if let Some(value) = *ctx.value {
    ///     e::div().text(value)
    /// } else {
    ///     e::div().text("Is none")
    /// }
    /// # }}
    /// ```
    /// The issue here is that the outer div (which might be a more expensive structure to create) is
    /// recreated everytime `value` changes, even if it is `Some(0) -> Some(1)`
    /// This is where you might reach for `ctx.watch`, and in fact that works perfectly:
    /// ```rust
    /// # use natrix::prelude::*;
    /// # #[derive(State)]
    /// # struct App {value: Signal<Option<u32>>}
    /// # fn render() -> impl Element<App> {
    /// # |mut ctx: RenderCtx<App>| {
    /// if ctx.watch(|ctx| ctx.value.is_some()) {
    ///     e::div().text(|ctx: RenderCtx<App>| ctx.value.unwrap())
    /// } else {
    ///     e::div().text("Is none")
    /// }
    /// # }}
    /// ```
    /// And this works, Now a change from `Some(0)` to `Some(1)` will only run the inner closure and
    /// the outer div is reused. but there is one downside, we need `.unwrap` because the inner closure is
    /// technically isolated, and this is ugly, and its easy to do by accident. and you might forget
    /// the outer condition.
    ///
    /// This is where guards come into play:
    /// ```rust
    /// # use natrix::prelude::*;
    /// # #[derive(State)]
    /// # struct App {value: Signal<Option<u32>>}
    /// # fn render() -> impl Element<App> {
    /// # |mut ctx: RenderCtx<App>| {
    /// if let Some(value_guard) = ctx.guard(lens!(App => .value).deref()) {
    ///     e::div().text(move |mut ctx: RenderCtx<App>| *ctx.get(value_guard))
    /// } else {
    ///     e::div().text("Is none")
    /// }
    /// # }}
    /// ```
    /// Here `value_guard` is actually not the value at all, its a lightweight value thats can be
    /// captured by child closures and basically is a way to say "I know that in this context this
    /// value is `Some`"
    ///
    /// Internally this uses `ctx.watch` and `.unwrap` (which should never fail)
    /// Guard also functions on `Result`
    #[inline]
    pub fn guard<L, G>(&mut self, lens: L) -> G::Output<L>
    where
        L: LensInner<Source = C, Target = G> + Clone,
        G: Guardable<C>,
    {
        let check_lens = lens.clone();
        let check = self.watch(
            #[inline]
            move |ctx| check_lens.clone().resolve(&mut ctx.ctx.data).check(),
        );
        G::into_lens(check, lens)
    }
}

/// A type that is guardable.
/// This generally means types such as `Option` and `Result`
// TODO: Make this derivable
pub trait Guardable<S>: Sized {
    /// The result of guarding this value.
    /// The generic is the source lens
    type Output<L: Lens<S, Self>>;

    /// The output of the `ctx.watch` call, this should indicate the variant we have
    type WatchState: PartialEq + Clone + 'static;

    /// The function that will be called in `ctx.watch`
    fn check(&self) -> Self::WatchState;

    /// Construct yourself wrapping a lens that will grab the corresponding variant data in a
    /// paniciky way
    fn into_lens<L>(check_result: Self::WatchState, to_you: L) -> Self::Output<L>
    where
        L: Lens<S, Self>;
}

/// A lens that calls `.unwrap` on a option value.
///
/// This lens is designed to be created by the `Guardable` trait, which ensures
/// it is only used when the `Option` is `Some`, preventing panics.
pub struct OptionLens<T>(PhantomData<T>);

impl<T> OptionLens<T> {
    /// Create a new one
    #[must_use]
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<T> Clone for OptionLens<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for OptionLens<T> {}

impl<T: 'static> LensInner for OptionLens<Option<T>> {
    type Source = Option<T>;
    type Target = T;

    #[expect(clippy::unreachable, reason = "Only constructable by guard")]
    #[inline]
    fn resolve(self, source: &mut Self::Source) -> &mut Self::Target {
        let Some(value) = source.as_mut() else {
            let msg = "OptionLens called on `None`";
            log::error!("{msg}");
            unreachable!("{msg}");
        };
        value
    }

    #[inline]
    fn resolve_failable(self, source: &mut Self::Source) -> Option<&mut Self::Target> {
        source.as_mut()
    }
}

impl<S, T> Guardable<S> for Option<T>
where
    T: 'static,
{
    type Output<L: Lens<S, Self>> = Option<lens::Chain<L, OptionLens<Self>>>;
    type WatchState = bool;

    fn check(&self) -> Self::WatchState {
        self.is_some()
    }

    fn into_lens<L>(check_result: Self::WatchState, to_you: L) -> Self::Output<L>
    where
        L: Lens<S, Self>,
    {
        if check_result {
            Some(to_you.then(OptionLens::new()))
        } else {
            None
        }
    }
}

/// A lens that calls `.unwrap()` on a `Result` to access the `Ok` value.
///
/// This lens is designed to be created by the `Guardable` trait, which ensures
/// it is only used when the `Result` is `Ok`, preventing panics.
pub struct OkLens<R>(PhantomData<R>);

impl<R> OkLens<R> {
    /// Creates a new `OkLens`.
    #[must_use]
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<T> Clone for OkLens<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for OkLens<T> {}

impl<T, E> LensInner for OkLens<Result<T, E>>
where
    T: 'static,
    E: 'static,
{
    type Source = Result<T, E>;
    type Target = T;

    #[expect(clippy::unreachable, reason = "Only constructable by guard")]
    #[inline]
    fn resolve(self, source: &mut Self::Source) -> &mut Self::Target {
        let Ok(value) = source.as_mut() else {
            let msg = "OkLens called on `Err`";
            log::error!("{msg}");
            unreachable!("{msg}");
        };
        value
    }

    #[inline]
    fn resolve_failable(self, source: &mut Self::Source) -> Option<&mut Self::Target> {
        source.as_mut().ok()
    }
}

/// A lens that calls `.unwrap_err()` on a `Result` to access the `Err` value.
///
/// This lens is designed to be created by the `Guardable` trait, which ensures
/// it is only used when the `Result` is `Err`, preventing panics.
pub struct ErrLens<R>(PhantomData<R>);

impl<R> ErrLens<R> {
    /// Creates a new `ErrLens`.
    #[must_use]
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<T> Clone for ErrLens<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for ErrLens<T> {}

impl<T, E> LensInner for ErrLens<Result<T, E>>
where
    T: 'static,
    E: 'static,
{
    type Source = Result<T, E>;
    type Target = E;

    #[expect(clippy::unreachable, reason = "Only constructable by guard")]
    #[inline]
    fn resolve(self, source: &mut Self::Source) -> &mut Self::Target {
        let Err(value) = source.as_mut() else {
            let msg = "ErrLens called on `Ok`";
            log::error!("{msg}");
            unreachable!("{msg}");
        };
        value
    }

    #[inline]
    fn resolve_failable(self, source: &mut Self::Source) -> Option<&mut Self::Target> {
        source.as_mut().err()
    }
}

impl<S, T, E> Guardable<S> for Result<T, E>
where
    T: 'static,
    E: 'static,
{
    type Output<L: Lens<S, Self>> =
        Result<lens::Chain<L, OkLens<Self>>, lens::Chain<L, ErrLens<Self>>>;
    type WatchState = bool;

    #[inline]
    fn check(&self) -> Self::WatchState {
        self.is_ok()
    }

    #[inline]
    fn into_lens<L>(check_result: Self::WatchState, to_you: L) -> Self::Output<L>
    where
        L: Lens<S, Self>,
    {
        if check_result {
            Ok(to_you.then(OkLens::new()))
        } else {
            Err(to_you.then(ErrLens::new()))
        }
    }
}
