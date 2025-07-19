//! Implements wrappers around various global statics for the reactivity system tracking.

use std::cell::{Cell, RefCell};

use smallvec::SmallVec;

use super::state::HookKey;
use crate::error_handling::{log_or_panic, log_or_panic_assert};
use crate::macro_ref::HookDepListIter;
use crate::reactivity::state::HookDepListHolder;

thread_local! {
    /// The current hook the signal is being accessed in.
    static CURRENT_HOOK: Cell<Option<HookKey>> = const {Cell::new(None)};
    /// List for signals to push deps lists into
    static DIRTY_HOOKS: RefCell<Option<HookDepListHolder>> = const {RefCell::new(None)};
}

/// Return the current hook if any
#[inline]
pub(crate) fn current_hook() -> Option<HookKey> {
    CURRENT_HOOK.get()
}

/// Run the given function with the given hook as the current hook
/// and restore the previous hook on completion.
#[inline]
pub(crate) fn with_hook<R>(new_hook: HookKey, func: impl FnOnce() -> R) -> R {
    let previous_hook = CURRENT_HOOK.replace(Some(new_hook));
    let result = func();
    CURRENT_HOOK.set(previous_hook);
    result
}

/// Push a iterator to the dirty hooks list
#[inline]
pub(crate) fn reg_dirty_list(calc: impl FnOnce() -> HookDepListIter) {
    DIRTY_HOOKS.with(|dirty_hooks| {
        let Ok(mut dirty_hooks) = dirty_hooks.try_borrow_mut() else {
            log_or_panic!("`DIRTY_HOOKS` overlapping borrow");
            return;
        };

        if let Some(dirty_hooks) = &mut *dirty_hooks {
            dirty_hooks.push(calc());
        }
    });
}

/// Drain the list of dirty hooks
#[inline]
pub(crate) fn with_dirty_tracking<R>(func: impl FnOnce() -> R) -> (HookDepListHolder, R) {
    DIRTY_HOOKS.with(|dirty_hooks| {
        let Ok(mut dirty_hooks) = dirty_hooks.try_borrow_mut() else {
            log_or_panic!("`DIRTY_HOOKS` overlapping borrow");
            return;
        };

        log_or_panic_assert!(
            dirty_hooks.is_none(),
            "`with_dirty_tracking` called recursively"
        );
        *dirty_hooks = Some(SmallVec::new());
    });

    let result = func();

    let dirty_list = DIRTY_HOOKS.with(|dirty_hooks| {
        let Ok(mut dirty_hooks) = dirty_hooks.try_borrow_mut() else {
            log_or_panic!("`DIRTY_HOOKS` overlapping borrow");
            return SmallVec::new();
        };

        let Some(dirty_hooks) = dirty_hooks.take() else {
            log_or_panic!("`DIRTY_HOOKS` gone after being set");
            return SmallVec::new();
        };

        dirty_hooks
    });

    (dirty_list, result)
}

/// Clear the statics
#[cfg(feature = "test_utils")]
pub(crate) fn clear() {
    DIRTY_HOOKS.set(None);
    CURRENT_HOOK.set(None);
}

#[cfg(test)]
#[expect(clippy::expect_used, clippy::unreachable, reason = "Its tests")]
mod tests {
    use indexmap::IndexSet;
    use slotmap::KeyData;

    use super::*;

    #[test]
    fn outside_closure_is_no_hook() {
        let hook1 = HookKey::from(KeyData::from_ffi(1));

        assert_eq!(current_hook(), None);
        with_hook(hook1, || {});
        assert_eq!(current_hook(), None);
    }

    #[test]
    fn setting_hook_gives_hook() {
        let hook1 = HookKey::from(KeyData::from_ffi(1));

        with_hook(hook1, || {
            assert_eq!(current_hook(), Some(hook1));
        });
    }

    #[test]
    fn nesting_hook() {
        let hook1 = HookKey::from(KeyData::from_ffi(1));
        let hook2 = HookKey::from(KeyData::from_ffi(2));

        with_hook(hook1, || {
            with_hook(hook2, || {
                assert_eq!(current_hook(), Some(hook2));
            });
            assert_eq!(current_hook(), Some(hook1));
        });
    }

    #[test]
    fn dirty_tracking() {
        let hook1 = HookKey::from(KeyData::from_ffi(1));
        let hook2 = HookKey::from(KeyData::from_ffi(2));

        let (mut result, ()) = with_dirty_tracking(|| {
            reg_dirty_list(|| {
                let mut keys = IndexSet::new();
                keys.insert(hook1);
                keys.insert(hook2);
                keys.into_iter()
            });
        });

        assert_eq!(result.len(), 1);
        let iter = result.first_mut().expect("No results in dirty tracking.");
        assert_eq!(iter.next(), Some(hook1));
        assert_eq!(iter.next(), Some(hook2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn reg_dirty_list_lazy() {
        reg_dirty_list(|| {
            unreachable!("Dirty list closure was called even tho no dirty tracking active.")
        });
    }
}
