//! Signals for tracking reactive depdencies and modifications.

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use crate::state::{ComponentData, HookKey, KeepAlive, State};

/// State passed to rendering callbacks
pub(crate) struct RenderingState<'s> {
    /// Push objects to this array to keep them alive as long as the parent context is valid.
    pub(crate) keep_alive: &'s mut Vec<KeepAlive>,
    /// The hooks that are a child of this
    pub(crate) hooks: &'s mut Vec<HookKey>,
    /// The parent render context, can be used to register it as a depdency of yourself
    pub(crate) parent_dep: HookKey,
}

/// A signal tracks reads and writes, as well as
pub struct Signal<T> {
    /// The data to be tracked.
    data: T,
    /// The flag for whether this signal has been written to
    written: bool,
    /// The flag for whether this signal has been read
    /// this is a `Cell` to allow for modification in `Deref`
    read: Cell<bool>,
    /// A hashset of the dependencies.
    ///
    /// Actually calling said depdencies is the responsibility of the `State` struct.
    /// Depdencies are also lazily removed by the `State` struct, and hence might contain stale
    /// pointers.
    deps: Vec<HookKey>,
}

/// The `written` and `read` flags of a signal extracted
#[derive(Copy, Clone)]
pub struct SignalState {
    /// Was the signal written to
    written: bool,
    /// Was the signal read
    read: bool,
}

impl<T: std::fmt::Debug> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T> Signal<T> {
    /// Create a new signal with the specified data
    pub fn new(data: T) -> Self {
        Self {
            data,
            written: false,
            read: Cell::new(false),
            deps: Vec::new(),
        }
    }

    #[doc(hidden)]
    pub fn pop_state(&mut self) -> SignalState {
        let result = SignalState {
            written: self.written,
            read: self.read.get(),
        };
        self.clear();
        result
    }

    #[doc(hidden)]
    pub fn set_state(&mut self, state: SignalState) {
        self.written = state.written;
        self.read.set(state.read);
    }
}

/// Methods for signals that arent generic over the contained data.
///
/// The use case of this trait is allowing the `State` struct
/// to work on a `Vec<&dyn SignalMethods<C>>`, which means the derive macro does not need to
/// generate a lot of a delegation methods.
///
/// Performance: This approach leads to cleaner-- less macro heavy --code, but does have a
/// performance hit via vtable and vector allocation overhead.
pub trait SignalMethods {
    /// Clear the `read` and `written` flags.
    fn clear(&mut self);
    /// Adds the given depedency to the hashset if the `read` flag is set.
    fn register_dep(&mut self, dep: HookKey);
    /// Return a mutable reference to the depdencies to facilitate efficent cleanup and
    /// deduplication in the `State struct`
    ///
    /// We are doing the cleaning in the `State` struct because it lets us deduplicate the changed
    /// hooks in `.update` without looping over the hashset twice.
    fn deps(&mut self) -> std::vec::Drain<'_, HookKey>;
    /// Return the value of the `written` field
    fn changed(&self) -> bool;
}

impl<T> SignalMethods for Signal<T> {
    fn clear(&mut self) {
        self.written = false;
        self.read.set(false);
    }

    fn register_dep(&mut self, dep: HookKey) {
        if self.read.get() {
            self.deps.push(dep);
        }
    }

    fn changed(&self) -> bool {
        self.written
    }

    fn deps(&mut self) -> std::vec::Drain<'_, HookKey> {
        self.deps.drain(..)
    }
}

impl<T> Deref for Signal<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.read.set(true);
        &self.data
    }
}
impl<T> DerefMut for Signal<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.written = true;
        &mut self.data
    }
}

/// All reactive hooks will implement this trait to allow them to be stored as `dyn` objects.
pub(crate) trait ReactiveHook<C: ComponentData> {
    /// Recalculate the hook and apply its update.
    ///
    /// Hooks should recall `ctx.reg_dep` with the you paramater to re-register any potential
    /// depdencies as the update method uses `.drain(..)` on depdencies (this is also to ensure
    /// reactive state that is only accesed in some conditions is recorded).
    fn update(&mut self, _ctx: &mut State<C>, _you: HookKey) -> UpdateResult;
    /// Drop keep alives and other state that will be invalidated in `update`
    fn drop_deps(&mut self) -> Option<std::vec::Drain<'_, HookKey>> {
        None
    }
}

/// The result of pre-update
pub(crate) enum UpdateResult {
    /// Do nothing extra
    Nothing,
    /// Run this hook after this one
    RunHook(HookKey),
}

/// Operations that are more ergonomic but inconsistent
#[cfg(feature = "ergonomic_ops")]
mod ergonomin_ops {
    use std::ops::{
        AddAssign,
        BitAndAssign,
        BitOrAssign,
        BitXorAssign,
        DivAssign,
        MulAssign,
        RemAssign,
        ShlAssign,
        ShrAssign,
        SubAssign,
    };

    use super::Signal;

    impl<R, T: PartialEq<R>> PartialEq<R> for Signal<T> {
        fn eq(&self, other: &R) -> bool {
            **self == *other
        }
    }

    impl<R, T: PartialOrd<R>> PartialOrd<R> for Signal<T> {
        fn partial_cmp(&self, other: &R) -> Option<std::cmp::Ordering> {
            (**self).partial_cmp(other)
        }
    }

    /// Generate inplace operations for signal
    macro_rules! inplace_op {
        ($trait:ident. $method:ident()) => {
            impl<R, T: $trait<R>> $trait<R> for Signal<T> {
                fn $method(&mut self, rhs: R) {
                    (**self).$method(rhs);
                }
            }
        };
    }

    inplace_op!(AddAssign.add_assign());
    inplace_op!(SubAssign.sub_assign());
    inplace_op!(MulAssign.mul_assign());
    inplace_op!(DivAssign.div_assign());
    inplace_op!(RemAssign.rem_assign());
    inplace_op!(BitAndAssign.bitand_assign());
    inplace_op!(BitOrAssign.bitor_assign());
    inplace_op!(BitXorAssign.bitxor_assign());
    inplace_op!(ShlAssign.shl_assign());
    inplace_op!(ShrAssign.shr_assign());
}

#[cfg(test)]
mod tests {
    use super::{Signal, SignalMethods};
    // We put signals in a struct to simulate the real usage pattern where they are always fields
    // in a &ref
    struct Holder<T>(Signal<T>);

    #[test]
    fn reading() {
        let foo = &Holder(Signal::new(10));
        assert_eq!(*foo.0, 10);
        assert!(foo.0.read.get());
    }

    #[test]
    fn modify() {
        let foo = &mut Holder(Signal::new(10));
        *foo.0 = 20;

        assert!(foo.0.changed());
        assert_eq!(*foo.0, 20);
    }

    #[test]
    fn debug() {
        let data = "Hello World";
        let foo = &Holder(Signal::new(data));

        assert_eq!(format!("{:?}", foo.0), format!("{data:?}"));

        assert!(foo.0.read.get());
    }

    #[cfg(feature = "ergonomic_ops")]
    mod ergonomic_ops {
        use super::*;

        #[test]
        fn eq() {
            let foo = &Holder(Signal::new(10));

            assert_eq!(foo.0, 10);
            assert_ne!(foo.0, 20);

            assert!(foo.0.read.get());
        }

        #[test]
        fn cmp() {
            let foo = &Holder(Signal::new(10));

            assert!(foo.0 > 5);
            assert!(foo.0 < 20);

            assert!(foo.0.read.get());
        }

        macro_rules! test_inplace {
        ($name:ident: $inital:literal $operation:tt $value:literal -> $expected:literal) => {
            #[test]
            fn $name() {
                let foo = &mut Holder(Signal::new($inital));

                foo.0 $operation $value;

                assert!(foo.0.changed());
                assert_eq!(foo.0, $expected);
            }
        };
    }

        test_inplace!(inplace_add: 10 += 5 -> 15);
        test_inplace!(inplace_sub: 10 -= 5 -> 5);
        test_inplace!(inplace_mul: 10 *= 4 -> 40);
        test_inplace!(inplace_div: 10 /= 5 -> 2);
        test_inplace!(inplace_mod: 12 %= 10 -> 2);
        test_inplace!(inplace_and: 0b1100 &= 0b1010 -> 0b1000);
        test_inplace!(inplace_or:  0b0100 |= 0b0010 -> 0b0110);
        test_inplace!(inplace_xor: 0b1100 ^= 0b1000 -> 0b0100);
        test_inplace!(inplace_shl: 1 <<= 1 -> 2);
        test_inplace!(inplace_shr: 4 >>= 1 -> 2);
    }
}
