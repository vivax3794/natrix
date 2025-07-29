//! Various traits and functions for writing reusable getter closures.
//! Most abstractions here are built around the `Ref` enum.

// TODO: (opt-in) Lint for non-maximally flexible closure forms.
// * `-> Ref<Option<T>>` if the `Option` variant is never changed (prefer `-> Option<Ref<T>>`)
// (same for Result)
// * `-> Ref<Signal<T>>` (prefer `-> Ref<T>`) (`Ref<T>` where `T` is a non-signal `State` is fine)
use std::ops::{Deref, DerefMut};

/// Either a `&T` or a `&mut T`
/// Use to provide generic getters.
///
///  INVARIANT: All closures dealing with these should preserve the enum variant.
/// Meaning a closure that wants to downgrade a reference needs to just return `&T` instead.
/// Natrix assumes all closure of the form `Fn(Ref<T>) -> Ref<R>` Maintain the variant given.
///
/// INVARIANT: `Read` and `Mut` must only be created in render hooks and event handlers.
/// *not* in async contexts or similar, as certain closures created by the framework assume sync
/// invaraints are upheld.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Ref<'a, T: ?Sized> {
    /// a `&T`
    Read(&'a T),
    /// a `&mut T`
    Mut(&'a mut T),
    /// a `Option<&mut T>` (used in async)
    FaillableMut(Option<&'a mut T>),
    // MAYBE: A owned variant?
}

impl<'a, T: ?Sized> Ref<'a, T> {
    /// Run a given function depending on whether its a `&` or `&mut`.
    /// These need to return the same type.
    // TODO: Lint against doing mutations in `write` (might be hard to detect.)
    #[inline]
    #[must_use]
    pub fn map<R: ?Sized>(
        self,
        read: impl FnOnce(&'a T) -> &'a R,
        write: impl FnOnce(&'a mut T) -> &'a mut R,
    ) -> Ref<'a, R> {
        match self {
            Ref::Read(value) => Ref::Read(read(value)),
            Ref::Mut(value) => Ref::Mut(write(value)),
            Ref::FaillableMut(None) => Ref::FaillableMut(None),
            Ref::FaillableMut(Some(value)) => Ref::FaillableMut(Some(write(value))),
        }
    }

    /// Project a value, meaning transform from `Ref<...>` to `Foo<Ref<...>>`
    /// ```rust
    /// # use natrix::prelude::*;
    /// fn foo(maybe_u8: Ref<Option<u8>>) {
    ///     if let Some(value) = maybe_u8.project() {
    ///         println!("{:?}", value.into_read());
    ///     }
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn project(self) -> T::Projected<'a>
    where
        T: Project,
    {
        T::project(self)
    }

    /// dereference the inner value
    #[inline]
    #[must_use]
    pub fn deref(self) -> Ref<'a, T::Target>
    where
        T: Deref + DerefMut,
    {
        self.map(|value| &**value, |value| &mut **value)
    }
}

impl<'a, T: ?Sized> From<&'a T> for Ref<'a, T> {
    #[inline]
    fn from(value: &'a T) -> Self {
        Ref::Read(value)
    }
}
impl<'a, T: ?Sized> From<&'a mut T> for Ref<'a, T> {
    #[inline]
    fn from(value: &'a mut T) -> Self {
        Ref::Mut(value)
    }
}

/// for example `Ref<Option<T>>` to `Option<Ref<T>>`, basically a abstraction over the various
/// `as_mut`/`as_ref` methods.
// TODO: Make this deriveble.
pub trait Project: Sized {
    /// The result of the projection, should contain `Ref`s with the `'a` lifetime.
    type Projected<'a>
    where
        Self: 'a;

    /// Project a `Ref<Self>` to `Self::Projected`
    fn project(value: Ref<'_, Self>) -> Self::Projected<'_>;
}

impl<T> Project for Option<T> {
    type Projected<'a>
        = Option<Ref<'a, T>>
    where
        Self: 'a;

    fn project(value: Ref<'_, Self>) -> Self::Projected<'_> {
        match value {
            Ref::Read(value) => value.as_ref().map(Into::into),
            Ref::Mut(value) => value.as_mut().map(Into::into),
            Ref::FaillableMut(None) => Some(Ref::FaillableMut(None)),
            Ref::FaillableMut(Some(value)) => {
                value.as_mut().map(|value| Ref::FaillableMut(Some(value)))
            }
        }
    }
}

impl<T, E> Project for Result<T, E> {
    type Projected<'a>
        = Result<Ref<'a, T>, Ref<'a, E>>
    where
        Self: 'a;

    fn project(value: Ref<'_, Self>) -> Self::Projected<'_> {
        match value {
            Ref::Read(value) => value.as_ref().map(Into::into).map_err(Into::into),
            Ref::Mut(value) => value.as_mut().map(Into::into).map_err(Into::into),
            Ref::FaillableMut(None) => Err(Ref::FaillableMut(None)),
            Ref::FaillableMut(Some(value)) => value
                .as_mut()
                .map(|val| Ref::FaillableMut(Some(val)))
                .map_err(|val| Ref::FaillableMut(Some(val))),
        }
    }
}

/// Trait for items that can be downgraded to references.
/// Specifically this must be implemented for *types* that can represent both mutable and immutable
/// references, i.e ones that build on `Ref`, a implementation with a constant `None` in `as_mut`
/// should be considered broken.
///
/// Note, to avoid unwraps in your code for this you can use `RefClosure` apis instead.
/// Which hides the unwrap behind the assumption the closure is well behaved (maintains variant.)
// MAYBE: Make derivable
pub trait Downgrade<'a> {
    /// The `&` version of this type.
    type ReadOutput;

    /// The `&mut` version of this type.
    type MutOutput;

    /// Convert this to a equivalent type with `&`,
    /// Will downgrade a `&mut` if needed.
    /// Might fail if given `Ref::FaillableMut(None)`
    fn into_read(self) -> Option<Self::ReadOutput>;
    /// Convert this to a equivalent type with `&mut`,
    /// Will return `None` if read variant.
    fn into_mut(self) -> Option<Self::MutOutput>;
}

impl<'a, T: ?Sized> Downgrade<'a> for Ref<'a, T> {
    type ReadOutput = &'a T;
    type MutOutput = &'a mut T;

    #[inline]
    fn into_read(self) -> Option<Self::ReadOutput> {
        match self {
            Ref::Read(value) => Some(value),
            Ref::Mut(value) => Some(value),
            Ref::FaillableMut(value) => value.map(|x| &*x),
        }
    }

    #[inline]
    fn into_mut(self) -> Option<Self::MutOutput> {
        match self {
            Ref::Read(_) => None,
            Ref::Mut(value) => Some(value),
            Ref::FaillableMut(value) => value,
        }
    }
}

// NOTE: We do not implement `Downgradable` for `&`
// Because a type that always fails to downgrade into `&mut` is not a valid `Downgradble`
impl<'a, T: ?Sized> Downgrade<'a> for &'a mut T {
    type ReadOutput = &'a T;
    type MutOutput = &'a mut T;
    #[inline]
    fn into_read(self) -> Option<Self::ReadOutput> {
        Some(self)
    }
    #[inline]
    fn into_mut(self) -> Option<Self::MutOutput> {
        Some(self)
    }
}
impl<'a, T> Downgrade<'a> for Option<T>
where
    T: Downgrade<'a>,
{
    type ReadOutput = Option<T::ReadOutput>;
    type MutOutput = Option<T::MutOutput>;

    fn into_read(self) -> Option<Self::ReadOutput> {
        let result = match self {
            None => None,
            Some(value) => Some(value.into_read()?),
        };
        Some(result)
    }
    fn into_mut(self) -> Option<Self::MutOutput> {
        match self {
            None => Some(None),
            Some(value) => value.into_mut().map(Some),
        }
    }
}
impl<'a, T, E> Downgrade<'a> for Result<T, E>
where
    T: Downgrade<'a>,
    E: Downgrade<'a>,
{
    type ReadOutput = Result<T::ReadOutput, E::ReadOutput>;
    type MutOutput = Result<T::MutOutput, E::MutOutput>;

    fn into_read(self) -> Option<Self::ReadOutput> {
        Some(match self {
            Ok(value) => Ok(value.into_read()?),
            Err(value) => Err(value.into_read()?),
        })
    }
    fn into_mut(self) -> Option<Self::MutOutput> {
        Some(match self {
            Ok(value) => Ok(value.into_mut()?),
            Err(value) => Err(value.into_mut()?),
        })
    }
}

/// A Ref closure is a closure that takes a `Ref` and return some downgradable value.
/// And allows calling them with normal references and getting normal references back.
///
/// You should generally not use this bounds, and instead opt for the `impl Fn...` syntax.
// TODO: Create lint against using `call_read` and `call_mut` in async context.
pub trait RefClosure<'a, I: ?Sized, T: Downgrade<'a>> {
    /// Call the read path of this closure.
    /// This will never fail
    ///
    /// INVARIANT: Must not be called from async, use `call_failable`
    fn call_read(&self, value: &'a I) -> T::ReadOutput;

    /// Call the mut part of this path.
    /// This will panic if the closure returns `Ref::Read` event if given a `Ref::Mut`
    /// (Which shouldnt happen for any well behaving implementation)
    ///
    /// INVARIANT: Must not be called from async, use `call_failable`
    fn call_mut(&self, value: &'a mut I) -> T::MutOutput;

    /// Call the mut part of this path, but return `None` if any earlier invariants (like guards),
    /// are no longer valid.
    fn call_failable(&self, value: &'a mut I) -> Option<T::MutOutput>;
}
impl<'a, I, T, F> RefClosure<'a, I, T> for F
where
    F: Fn(Ref<'a, I>) -> T,
    T: Downgrade<'a>,
    I: 'a + ?Sized,
{
    #[expect(clippy::unreachable, reason = "Core invariant.")]
    fn call_read(&self, value: &'a I) -> T::ReadOutput {
        if let Some(value) = self(Ref::Read(value)).into_read() {
            value
        } else {
            unreachable!("Closure didnt return Read compatible result when given `Ref::Read`");
        }
    }

    #[expect(clippy::unreachable, reason = "Core invariant.")]
    fn call_mut(&self, value: &'a mut I) -> T::MutOutput {
        if let Some(value) = self(Ref::Mut(value)).into_mut() {
            value
        } else {
            unreachable!("Closure didnt return Mut compatible result when given `Ref::Mut`");
        }
    }

    fn call_failable(&self, value: &'a mut I) -> Option<T::MutOutput> {
        self(Ref::FaillableMut(Some(value))).into_mut()
    }
}

/// A "alias trait" for `impl Fn(Ref<S>) -> Ref<R> + Clone + 'static`
pub trait Getter<S: ?Sized, R: ?Sized>:
    for<'a> Fn(Ref<'a, S>) -> Ref<'a, R> + Clone + 'static
{
}
impl<S, R, F> Getter<S, R> for F
where
    R: ?Sized,
    S: ?Sized,
    F: Clone + 'static,
    F: for<'a> Fn(Ref<'a, S>) -> Ref<'a, R>,
{
}

/// Access fields on a `Ref<T>`.
/// `field!(foo.bar.abc)` is equivalent to `foo.map(|foo| &foo.bar.abc, |foo| &mut foo.bar.abc)`
///
/// If you wish to use an expression for the target `Ref` use `()`
/// `field!((some_expression()).bar.abc)`
#[macro_export]
macro_rules! field {
    ($name:ident. $($field:ident).+) => {
        $name.map(|$name| &$name.$($field).+, |$name| &mut $name.$($field).+)
    };
    (($value:expr). $($field:ident).+) => {
        ($value).map(|value| &value.$($field).+, |value| &mut value.$($field).+)
    };
}

/// Clone the given values to be captured by the given closure.
///
/// `with!(move foo |...| ...)` is the same as `let foo = foo.clone(); move |...| ...`
/// For multiple captures use `()` like `with!(move (foo, bar) |...| ...)`
#[macro_export]
macro_rules! with {
    (
        move ($($arg:ident),*)
        $($closure:tt)+
    ) =>{{
        $(
            let $arg = $arg.clone();
        )*
        move $($closure)+
    }};
    (
        move $arg:ident
        $($closure:tt)+
    ) =>{{
        $crate::with!(move ($arg) $($closure)*)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Foo {
        value: u8,
    }

    #[test]
    fn map_read_keeps_read() {
        let x = Foo { value: 10 };
        let y = Ref::Read(&x).map(|val| &val.value, |val| &mut val.value);
        assert_eq!(y, Ref::Read(&x.value));
    }

    #[test]
    fn map_mut_keeps_mut() {
        let mut x = Foo { value: 10 };
        let y = Ref::Mut(&mut x).map(|val| &val.value, |val| &mut val.value);
        assert_eq!(y.into_mut(), Some(&mut 10));
    }

    #[test]
    fn field_direct() {
        let x = Foo { value: 10 };
        let borrow = Ref::Read(&x);
        let value = field!(borrow.value);
        assert_eq!(value.into_read(), Some(&10));
    }

    #[test]
    fn field_expr() {
        let x = Foo { value: 10 };
        let value = field!((Ref::Read(&x)).value);
        assert_eq!(value.into_read(), Some(&10));
    }

    fn identify(func: impl Fn(Ref<u8>) -> Ref<u8>) -> impl Fn(Ref<u8>) -> Ref<u8> {
        func
    }

    #[test]
    fn call_read() {
        let getter = identify(|value| value);
        let x = 10;
        assert_eq!(getter.call_read(&x), &10);
    }

    #[test]
    fn call_mut() {
        let getter = identify(|value| value);
        let mut x = 10;
        *getter.call_mut(&mut x) += 10;
        assert_eq!(x, 20);
    }
}
