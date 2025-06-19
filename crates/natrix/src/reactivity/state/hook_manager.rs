//! Handles state hooks

use slotmap::{SecondaryMap, SlotMap, new_key_type};

use super::State;
use crate::Component;
use crate::error_handling::log_or_panic;
use crate::reactivity::render_callbacks::ReactiveHook;

new_key_type! { pub(crate) struct HookKey; }

/// A manager for storing hooks
pub(crate) struct HookStore<T: Component> {
    /// The hooks themself
    hooks: SlotMap<HookKey, Box<dyn ReactiveHook<T>>>,
    /// The insertion order
    insertion_order: SecondaryMap<HookKey, u64>,
    /// The next key in the insertion order
    next_insertion_order: u64,
}

impl<T: Component> HookStore<T> {
    /// Create a new hook store
    pub(super) fn new() -> Self {
        Self {
            hooks: SlotMap::default(),
            insertion_order: SecondaryMap::default(),
            next_insertion_order: 0,
        }
    }

    /// Insert a hook
    pub(crate) fn insert_hook(&mut self, hook: Box<dyn ReactiveHook<T>>) -> HookKey {
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
            *slot = hook;
        }
    }

    /// insert a deummy and return the key
    pub(crate) fn insert_dummy(&mut self) -> HookKey {
        self.insert_hook(Box::new(crate::reactivity::render_callbacks::DummyHook))
    }

    /// Get the insertion order of the given key
    pub(super) fn insertion_order(&self, key: HookKey) -> Option<u64> {
        self.insertion_order.get(key).copied()
    }

    /// Drop all children of the hook
    pub(super) fn drop_hook(&mut self, hook: HookKey) {
        if let Some(hook) = self.hooks.remove(hook) {
            let mut hooks = hook.drop_us();
            for hook in hooks.drain(..) {
                self.drop_hook(hook);
            }
        }
    }
}

impl<T: Component> State<T> {
    /// Remove the hook from the slotmap, runs the function on it, then puts it back.
    ///
    /// This is to allow mut access to both the hook and self, which is required by most hooks.
    /// (and yes hooks also mutable access the slotmap while running)
    pub(super) fn run_with_hook_and_self<F, R>(&mut self, hook: HookKey, func: F) -> Option<R>
    where
        F: FnOnce(&mut Self, &mut Box<dyn ReactiveHook<T>>) -> R,
    {
        let slot_ref = self.hooks.hooks.get_mut(hook)?;
        let mut temp_hook: Box<dyn ReactiveHook<T>> =
            Box::new(crate::reactivity::render_callbacks::DummyHook);
        std::mem::swap(slot_ref, &mut temp_hook);

        let res = func(self, &mut temp_hook);

        let slot_ref = self.hooks.hooks.get_mut(hook)?;
        *slot_ref = temp_hook;

        Some(res)
    }
}
