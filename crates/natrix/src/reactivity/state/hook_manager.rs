//! Handles state hooks

use slotmap::{SecondaryMap, SlotMap, new_key_type};

use super::InnerCtx;
use crate::error_handling::log_or_panic;
use crate::reactivity::State;
use crate::reactivity::render_callbacks::ReactiveHook;

new_key_type! {
    #[doc(hidden)]
    pub struct HookKey;
}

/// A manager for storing hooks
pub(crate) struct HookStore<T: State> {
    /// The hooks themself
    /// NOTE: The `None` case is for yet to be initialized hooks, *not* for removed hooks.
    hooks: SlotMap<HookKey, Option<Box<dyn ReactiveHook<T>>>>,
    /// The insertion order
    pub(super) insertion_order: SecondaryMap<HookKey, u64>,
    /// The next key in the insertion order
    next_insertion_order: u64,
}

impl<T: State> HookStore<T> {
    /// Create a new hook store
    pub(super) fn new() -> Self {
        Self {
            hooks: SlotMap::default(),
            insertion_order: SecondaryMap::default(),
            next_insertion_order: 0,
        }
    }

    /// Insert a hook
    fn insert_hook(&mut self, hook: Option<Box<dyn ReactiveHook<T>>>) -> HookKey {
        let key = self.hooks.insert(hook);
        self.insertion_order.insert(key, self.next_insertion_order);

        self.next_insertion_order = if let Some(value) = self.next_insertion_order.checked_add(1) {
            value
        } else {
            log_or_panic!("Insertion order overflow");
            0
        };
        key
    }

    /// Update the value for a hook
    pub(crate) fn set_hook(&mut self, key: HookKey, hook: Box<dyn ReactiveHook<T>>) {
        if let Some(slot) = self.hooks.get_mut(key) {
            *slot = Some(hook);
        } else {
            log_or_panic!("Attempted to update missing key {key:?}");
        }
    }

    /// insert a deummy and return the key
    /// INVARIANT: Hooks must call `.reserve_key` in the relative order they are required to be updated and invalidated.
    pub(crate) fn reserve_key(&mut self) -> HookKey {
        self.insert_hook(None)
    }

    /// Drop all children of the hook
    pub(super) fn drop_hook(&mut self, hook_key: HookKey) {
        if let Some(hook) = self.hooks.remove(hook_key) {
            let Some(hook) = hook else {
                log_or_panic!("Attempted to drop `None` (uninitlized) hook, {hook_key:?}");
                return;
            };

            let mut hooks = hook.drop_us();
            for hook in hooks.drain(..) {
                self.drop_hook(hook);
            }
        }
    }
}

impl<T: State> InnerCtx<T> {
    /// Remove the hook from the slotmap, runs the function on it, then puts it back.
    ///
    /// This is to allow mut access to both the hook and self, which is required by most hooks.
    /// (and yes hooks also mutable access the slotmap while running)
    pub(super) fn run_with_hook_and_self<F, R>(&mut self, hook_key: HookKey, func: F) -> Option<R>
    where
        F: FnOnce(&mut Self, &mut Box<dyn ReactiveHook<T>>) -> R,
    {
        let slot_ref = self.hooks.hooks.get_mut(hook_key)?;
        let Some(mut hook) = slot_ref.take() else {
            log_or_panic!("Attempted to run `None` (Uninitialized) hook, {hook_key:?}");
            return None;
        };

        let res = func(self, &mut hook);
        self.hooks.set_hook(hook_key, hook);

        Some(res)
    }
}
