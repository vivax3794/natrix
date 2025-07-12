//! Lenses provide a abstraction around getter closures.

use std::marker::PhantomData;
use std::ops::DerefMut;

// TODO: Owned lens.

/// A lens provides a type safe way to access a sub-section of a State
pub trait LensInner: Clone + 'static {
    /// The input type to this lens
    type Source;
    /// The result of this lens
    type Target;

    /// Execute this lens
    fn resolve(self, source: &mut Self::Source) -> &mut Self::Target;
}

/// Marker trait for lenses that are safe to use in async.
///
/// Lenses that arent safe are lenses that might panic due to checks no longer being valid.
/// Such as lenses from `.guard`
// TODO: How to deal with optional data in async better.
// If you know its gonna be optional `Lens<..., Option<...>>` isnt that hard to do.
// But it doesnt generlize well, like if your state is `Result`, and a component library expects
// `Option` there isnt much you can do. I think what we want is a `FailableLens`?
// Which would return `Option<&mut T>` instead of forcing `&mut T`.
// It would let us say convert a `Lens<..., Option<T>>` into a `FailableLens<..., T>`.
// And let you then add lenses to the FailableLens.
// and if you have a `FailableLens<..., Option<T>>` you can flatten it.
// so like `lens!(App => .maybe).failable().then(lens!(Book => .maybe_author)).flatten()`
// would produce `FailableLens<App, String>`
// In this case maybe we should instead require that lenses in async are always `FailableLenses`?
// What if we want to have a guard, but then want to pass it into async.
// Can we somehow allow any `Lens` to make it self Failable?
// So if you can take a `impl Lens<App, String>` and in async it would always return
// `Option<String>` magically knowing if there was a `OptionLens` in the..
// Hmm I think we might be able to add like a `resolve_failable` to the `LensInner` type?
// That might actually work perfectly!
#[diagnostic::on_unimplemented(
    message = "This lens is not `AsyncSafe`",
    note = "Not all lenses are safe to use in async, such as those created by `ctx.guard`"
)]
pub trait AsyncSafe {}

impl<S: crate::reactivity::State> crate::reactivity::EventCtx<'_, S> {
    /// Execute a lens on this state
    #[inline]
    pub fn get<L: LensInner<Source = S>>(&mut self, lens: L) -> &mut L::Target {
        lens.resolve(&mut self.0.data)
    }
}

impl<S: crate::reactivity::State> crate::reactivity::RenderCtx<'_, '_, S> {
    /// Execute a lens on this state
    #[inline]
    pub fn get<L: LensInner<Source = S>>(&mut self, lens: L) -> &L::Target {
        lens.resolve(&mut self.ctx.data)
    }
}

#[cfg(feature = "async")]
impl<S: crate::reactivity::State> crate::reactivity::state::AsyncCtx<'_, S> {
    /// Execute a lens on this state
    #[inline]
    pub fn get<L: LensInner<Source = S> + AsyncSafe>(&mut self, lens: L) -> &mut L::Target {
        lens.resolve(&mut self.0.data)
    }
}

/// A lens thats just a direct function call.
pub struct Direct<F, S, T, const ASYNC_SAFE: bool = false> {
    /// The function to call
    pub func: F,
    /// Make rust happy
    phantom: PhantomData<(S, T)>,
}

impl<F: Copy, S, T, const ASYNC_SAFE: bool> Copy for Direct<F, S, T, ASYNC_SAFE> {}

impl<F: Clone, S, T, const ASYNC_SAFE: bool> Clone for Direct<F, S, T, ASYNC_SAFE> {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone(),
            phantom: PhantomData,
        }
    }
}

impl<F, S, T, const ASYNC_SAFE: bool> Direct<F, S, T, ASYNC_SAFE>
where
    F: Fn(&mut S) -> &mut T,
{
    /// Create a new direct lens
    ///
    /// You should generally prefer the `lens!` macro.
    pub fn new(func: F) -> Self {
        Direct {
            func,
            phantom: PhantomData,
        }
    }
}

impl<S, T, F, const ASYNC_SAFE: bool> LensInner for Direct<F, S, T, ASYNC_SAFE>
where
    F: Fn(&mut S) -> &mut T,
    F: Clone + 'static,
    S: 'static,
    T: 'static,
{
    type Source = S;
    type Target = T;

    #[inline]
    fn resolve(self, source: &mut S) -> &mut T {
        (self.func)(source)
    }
}

impl<F, S, T> AsyncSafe for Direct<F, S, T, true> {}

/// Chain two lenses
#[derive(Clone, Copy)]
pub struct Chain<L1, L2>(pub L1, pub L2);

impl<L1, L2> LensInner for Chain<L1, L2>
where
    L1: LensInner,
    L2: LensInner<Source = L1::Target>,
    L1::Target: 'static,
{
    type Source = L1::Source;
    type Target = L2::Target;

    #[inline]
    fn resolve(self, source: &mut Self::Source) -> &mut Self::Target {
        self.1.resolve(self.0.resolve(source))
    }
}
impl<L1: AsyncSafe, L2: AsyncSafe> AsyncSafe for Chain<L1, L2> {}

/// A Lens that calls `.deref` on the value
pub struct DerefLens<T>(pub PhantomData<T>);

impl<T> Copy for DerefLens<T> {}
impl<T> Clone for DerefLens<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Default for DerefLens<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> LensInner for DerefLens<T>
where
    T: DerefMut + 'static,
    T::Target: Sized,
{
    type Source = T;
    type Target = T::Target;

    #[inline]
    fn resolve(self, source: &mut Self::Source) -> &mut Self::Target {
        &mut *source
    }
}

impl<T: AsyncSafe> AsyncSafe for DerefLens<T> {}

/// A lens provides a type safe way to access a sub-section of a State
pub trait Lens<S, T>: LensInner<Source = S, Target = T> {
    /// Chain a lens
    #[inline]
    fn then<L>(self, other: L) -> Chain<Self, L>
    where
        Self: Sized,
        L: LensInner<Source = Self::Target>,
    {
        Chain(self, other)
    }

    /// Map a function on the lens
    /// You should generally prefer the `.then` method + the `lens!` macro.
    #[inline]
    fn map<F, R>(self, func: F) -> Chain<Self, Direct<F, Self::Target, R, false>>
    where
        Self: Sized,
        F: Fn(&mut Self::Target) -> &mut R,
    {
        Chain(self, Direct::new(func))
    }

    /// Map a function on the lens, but keeping it `AsyncSafe`.
    /// This means you can not use any surrounding state assumptions, such as `ctx.watch`
    /// to perform potentially panicy operations.
    ///
    /// You should generally prefer the `.then` method + the `lens!` macro.
    #[inline]
    fn map_assert_async_safe<F, R>(self, func: F) -> Chain<Self, Direct<F, Self::Target, R, true>>
    where
        Self: Sized,
        F: Fn(&mut Self::Target) -> &mut R,
    {
        Chain(self, Direct::new(func))
    }

    /// Deref the value the lens is pointing at, this is often required as the final step for
    /// getting at a signal.
    #[inline]
    fn deref(self) -> Chain<Self, DerefLens<T>> {
        Chain(self, DerefLens::default())
    }
}

impl<S, T, Ty> Lens<S, T> for Ty where Ty: LensInner<Source = S, Target = T> {}

/// A `Direct` lens to access a specific field, or sub-field
#[macro_export]
macro_rules! lens {
    ($source:ty => $(. $field:ident)+) => {
        $crate::lens::Direct::<_, _, _, true>::new(
            #[inline]
            |value: &mut $source| &mut value$(.$field)+
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lens;

    struct Book {
        title: String,
    }

    struct Books {
        rust: Book,
        natrix: Book,
    }

    fn test_data() -> Books {
        Books {
            rust: Book {
                title: String::from("The rust book"),
            },
            natrix: Book {
                title: String::from("Natrix Guide"),
            },
        }
    }

    #[test]
    fn test_direct() {
        let mut books = test_data();
        let lens = lens!(Books => .rust.title);
        assert_eq!(lens.resolve(&mut books), "The rust book");
    }

    #[test]
    fn test_then() {
        let mut books = test_data();

        let title_lens = lens!(Book => .title);
        let rust_lens = lens!(Books => .rust);
        let natrix_lens = lens!(Books => .natrix);

        assert_eq!(
            rust_lens.then(title_lens).resolve(&mut books),
            "The rust book"
        );
        assert_eq!(
            natrix_lens.then(title_lens).resolve(&mut books),
            "Natrix Guide"
        );
    }
}
