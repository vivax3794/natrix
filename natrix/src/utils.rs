//! Utility traits and structs

/// A version of the stdlib `Any` with no type ids, downcasting, etc
/// Its the minimal possible dyn object, mainly used for keep alive.
pub(crate) trait SmallAny {}
impl<T> SmallAny for T {}

use std::hash::Hash;
use std::rc::{Rc, Weak};

/// A hashset using the the no-hash hasher, our primary hashset usage is pointers, which are unqiue
/// enough
#[allow(clippy::disallowed_types)]
pub(crate) type HashSet<T> = std::collections::HashSet<T, nohash_hasher::BuildNoHashHasher<T>>;

/// Hash and Compare a `Rc` based on its pointer.
/// Used for reactive hooks for deduplication.
#[derive(Debug)]
pub(crate) struct RcCmpPtr<T>(pub Rc<T>);

impl<T> Clone for RcCmpPtr<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> PartialEq for RcCmpPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for RcCmpPtr<T> {}

impl<T> Hash for RcCmpPtr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr_value = Rc::as_ptr(&self.0) as usize;
        state.write_usize(ptr_value);
    }
}

/// Hash and Compare a `Weak` based on its pointer.
/// Used for reactive hooks for deduplication.
#[derive(Debug)]
pub(crate) struct WeakCmpPtr<T>(pub Weak<T>);
impl<T> Clone for WeakCmpPtr<T> {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl<T> PartialEq for WeakCmpPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for WeakCmpPtr<T> {}

impl<T> Hash for WeakCmpPtr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr_value = Weak::as_ptr(&self.0) as usize;
        state.write_usize(ptr_value);
    }
}

impl<T> nohash_hasher::IsEnabled for WeakCmpPtr<T> {}
impl<T> nohash_hasher::IsEnabled for RcCmpPtr<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct NoCmp;

    #[test]
    fn eq_rc() {
        let foo = RcCmpPtr(Rc::new(NoCmp));
        let bar = RcCmpPtr(Rc::clone(&foo.0));

        assert_eq!(foo, bar);
    }

    #[test]
    fn ne_rc() {
        let foo = RcCmpPtr(Rc::new(NoCmp));
        let bar = RcCmpPtr(Rc::new(NoCmp));

        assert_ne!(foo, bar);
    }

    #[test]
    fn hash_eq_rc() {
        let foo = RcCmpPtr(Rc::new(NoCmp));
        let bar = RcCmpPtr(Rc::clone(&foo.0));

        let mut set = HashSet::default();
        set.insert(foo);
        set.insert(bar);

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn hash_ne_rc() {
        let foo = RcCmpPtr(Rc::new(NoCmp));
        let bar = RcCmpPtr(Rc::new(NoCmp));

        let mut set = HashSet::default();
        set.insert(foo);
        set.insert(bar);

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn eq_weak() {
        let foo = WeakCmpPtr(Rc::downgrade(&Rc::new(NoCmp)));
        let bar = WeakCmpPtr(Weak::clone(&foo.0));

        assert_eq!(foo, bar);
    }

    #[test]
    fn ne_weak() {
        let foo = WeakCmpPtr(Rc::downgrade(&Rc::new(NoCmp)));
        let bar = WeakCmpPtr(Rc::downgrade(&Rc::new(NoCmp)));

        assert_ne!(foo, bar);
    }

    #[test]
    fn hash_eq_weak() {
        let foo = WeakCmpPtr(Rc::downgrade(&Rc::new(NoCmp)));
        let bar = WeakCmpPtr(Weak::clone(&foo.0));

        let mut set = HashSet::default();
        set.insert(foo);
        set.insert(bar);

        assert_eq!(set.len(), 1);
    }

    #[test]
    fn hash_ne_weak() {
        let foo = WeakCmpPtr(Rc::downgrade(&Rc::new(NoCmp)));
        let bar = WeakCmpPtr(Rc::downgrade(&Rc::new(NoCmp)));

        let mut set = HashSet::default();
        set.insert(foo);
        set.insert(bar);

        assert_eq!(set.len(), 2);
    }
}
