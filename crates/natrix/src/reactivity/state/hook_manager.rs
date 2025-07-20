//! Handles state hooks

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use super::InnerCtx;
use crate::error_handling::{log_or_panic, log_or_panic_assert};
use crate::reactivity::State;
use crate::reactivity::render_callbacks::ReactiveHook;

// MAYBE: feature flag to extend to larger types

/// The slot of the key
/// This is the number of concurrent hooks we can have.
/// atm around 65k
type KeySlot = u16;

/// The version in a slot, used to detect stale keys.
/// This * `KeySlot` is the number of hooks we can have in the lifetime of the program
/// (including deallocated ones).
/// atm around 4 million.
type KeyVersion = u16;

/// The type used to store the global insertion order.
/// Importantly this should have the same size as `HookKey`,
/// **but is NOT convertible between them**
/// This is because we will never need more insertion orders than possible hooks.
/// but they are fundamentally different attributes of a key.
pub(crate) type InsertionOrder = u32;

/// A key into a slotmap
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct HookKey {
    /// The slot to use
    pub(crate) slot: KeySlot,
    /// Version used to avoid use after free
    pub(crate) version: KeyVersion,
}

impl HookKey {
    /// A fallback key for error paths
    fn fallback() -> Self {
        Self {
            slot: KeySlot::MAX,
            version: KeyVersion::MAX,
        }
    }
}

/// The value of a slot
enum SlotValue<T> {
    /// The slot doesnt contain a value
    Empty,
    /// The slot is in use, but the value is moved out atm
    InUse,
    /// The slot is reserved
    Reserved {
        /// The insertion order of the reserved slot
        order: InsertionOrder,
    },
    /// The slot is occupied
    Occupied {
        /// The hook
        hook: Box<dyn ReactiveHook<T>>,
        /// The insertion order
        order: InsertionOrder,
    },
}

impl<T> Default for SlotValue<T> {
    fn default() -> Self {
        Self::Empty
    }
}

/// A slot in the slotmap
struct Slot<T> {
    /// The version of the slot
    version: KeyVersion,
    /// The value in the slot
    value: SlotValue<T>,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            version: 0,
            value: SlotValue::default(),
        }
    }
}

/// A manager for storing hooks
pub(crate) struct HookStore<T: State> {
    /// The hooks themself
    // PERF: Vector does not need len/capactiy of `usize`, can use `KeySlot`
    // PERF: Vector doesnt need to store len, we can always just fully fill with with null pointers
    hooks: Vec<Slot<T>>,
    /// The free spots in the hooks.
    // PERF: We dont need `Empty` variant in theory
    free: Vec<KeySlot>,
    /// The next key in the insertion order
    next_insertion_order: InsertionOrder,
}

impl<T: State> HookStore<T> {
    /// Create a new hook store
    pub(super) fn new() -> Self {
        Self {
            hooks: Vec::with_capacity(100),
            free: Vec::with_capacity(10),
            next_insertion_order: 0,
        }
    }

    /// reserve a hook key
    /// INVARIANT: Hooks must call `.reserve_key` in the relative order they are required to be updated and invalidated.
    pub(crate) fn reserve_key(&mut self) -> HookKey {
        let insertion_order = self.next_insertion_order;
        self.next_insertion_order = self.next_insertion_order.checked_add(1).unwrap_or_else(|| {
            log_or_panic!("Insertion order overflowed");
            // This is a very rare case, but restarting from zero should mean new hooks generally keep working.
            0
        });

        if let Some(slot) = self.free.pop() {
            let Some(entry) = self.hooks.get_mut(slot as usize) else {
                log_or_panic!("Value in free list out of bounds");
                return HookKey::fallback();
            };

            log_or_panic_assert!(
                matches!(entry.value, SlotValue::Empty),
                "Free slotmap wasnt `Empty`"
            );

            let Some(new_version) = entry.version.checked_add(1) else {
                log_or_panic!("Slot at max version was in free list");
                return self.reserve_key();
            };
            entry.version = new_version;

            entry.value = SlotValue::Reserved {
                order: insertion_order,
            };

            HookKey {
                slot,
                version: new_version,
            }
        } else {
            let Ok(slot) = self.hooks.len().try_into() else {
                log_or_panic!("Ran out of space in hooks slotmap");
                self.release_fallback_reclaim_high_versions();
                return self.reserve_key();
            };

            self.hooks.push(Slot {
                version: 0,
                value: SlotValue::Reserved {
                    order: insertion_order,
                },
            });
            HookKey { slot, version: 0 }
        }
    }

    /// Marks all `Empty` slots as free even if the their version number is at `MAX`
    /// This case is extremely rare, and this recovery path is good enough in most cases.
    fn release_fallback_reclaim_high_versions(&mut self) {
        for (index, slot) in self.hooks.iter_mut().enumerate() {
            if matches!(slot.value, SlotValue::Empty) {
                slot.version = 0;
                if let Ok(index) = index.try_into() {
                    self.free.push(index);
                } else {
                    log_or_panic!("Vec index overflows slot");
                }
            }
        }
    }

    /// Update the value for a hook
    pub(crate) fn set_hook(&mut self, key: HookKey, hook: Box<dyn ReactiveHook<T>>) {
        if let Some(slot) = self.hooks.get_mut(key.slot as usize) {
            // `set_hook` is always used directly after hook creation.
            log_or_panic_assert!(
                key.version == slot.version,
                "Mismatched version between key and slot in `set_hook`"
            );

            if let SlotValue::Reserved { order } = slot.value {
                slot.value = SlotValue::Occupied { hook, order };
            } else {
                log_or_panic!("Target slot wasnt reserved");
            }
        } else {
            log_or_panic!("Attempted to update missing slot {}", key.slot);
        }
    }

    /// Drop the hook and all of its children
    pub(super) fn drop_hook(&mut self, hook_key: HookKey) {
        let mut hooks_to_drop = vec![hook_key];
        while let Some(hook_key) = hooks_to_drop.pop() {
            if let Some(slot) = self.hooks.get_mut(hook_key.slot as usize) {
                if slot.version != hook_key.version {
                    continue;
                }

                self.free.push(hook_key.slot);
                match std::mem::take(&mut slot.value) {
                    SlotValue::Empty | SlotValue::InUse => {}
                    SlotValue::Reserved { .. } => {
                        log_or_panic!("Attempted to drop reserved hook.");
                    }
                    SlotValue::Occupied { hook, .. } => {
                        hooks_to_drop.extend(hook.drop_us());
                    }
                }
            } else {
                log_or_panic!("Attempted to drop hook outside current allocated index.");
            }
        }
    }

    /// Get the insertion order at a given hook, returns `None` if hook doesnt exist.
    pub(crate) fn get_insertion_order(&self, key: HookKey) -> Option<InsertionOrder> {
        if let Some(slot) = self.hooks.get(key.slot as usize) {
            if slot.version != key.version {
                return None;
            }

            match slot.value {
                SlotValue::Empty | SlotValue::InUse => None,
                SlotValue::Occupied { order, .. } | SlotValue::Reserved { order } => Some(order),
            }
        } else {
            log_or_panic!("hook key out of bounds.");
            None
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
        let Some(slot_ref) = self.hooks.hooks.get_mut(hook_key.slot as usize) else {
            log_or_panic!("HookKey outside bounds of slotmap");
            return None;
        };
        if slot_ref.version != hook_key.version {
            log::trace!("Version mismatch in `run_with_hook_and_self`");
            return None;
        }

        let mut slot_value = SlotValue::InUse;
        std::mem::swap(&mut slot_value, &mut slot_ref.value);

        let (order, mut hook) = match slot_value {
            SlotValue::Empty => {
                return None;
            }
            SlotValue::InUse => {
                log_or_panic!("Re-entry in `run_with_hook_and_self`");
                return None;
            }
            SlotValue::Reserved { .. } => {
                log_or_panic!("`run_with_hook_and_self` hit reserved hook");
                return None;
            }
            SlotValue::Occupied { hook, order } => (order, hook),
        };

        let res = func(self, &mut hook);

        let Some(slot_ref) = self.hooks.hooks.get_mut(hook_key.slot as usize) else {
            log_or_panic!("HookKey outside bounds of slotmap");
            return None;
        };

        if matches!(slot_ref.value, SlotValue::InUse) {
            slot_ref.value = SlotValue::Occupied { hook, order };
        } else {
            log_or_panic_assert!(
                matches!(slot_ref.value, SlotValue::Empty),
                "Slotmap entry overwritten in `run_with_hook_and_self`"
            );
        }

        Some(res)
    }
}

/// A linked list for holding signal dependencies.
/// Allows O(1) removal, and keeps insertion order
/// While removing stale entries based on slotmap.
///
/// Worst case this is bound to the size of the max amount of concurrent hooks.
/// More likely it will efficiently re-use memory even if rarely drained.
pub(crate) struct SignalDepList {
    /// The allocations of the nodes themself
    nodes: Vec<SignalDepNode>,
    /// The start of the list
    head: Option<usize>,
    /// The end of the list
    tail: Option<usize>,
    /// mapping from slot to index.
    items: HashMap<KeySlot, usize>,
}

/// A node in the linked list
struct SignalDepNode {
    /// The actual full hookkey
    key: HookKey,
    /// The index of the previous node
    previous: Option<usize>,
    /// The index of the next node
    next: Option<usize>,
}

impl SignalDepList {
    /// Create a new empty signal dep list
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            head: None,
            tail: None,
            items: HashMap::new(),
        }
    }

    /// Get the amount of current hooks (including stale ones.)
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Insert a new key into the linked list.
    /// This re-uses nodes with matching slots, as well as ensures.
    pub(crate) fn insert(&mut self, key: HookKey) {
        match self.items.entry(key.slot) {
            Entry::Vacant(entry) => {
                let new_index = self.nodes.len();
                let node = SignalDepNode {
                    key,
                    previous: self.tail,
                    next: None,
                };
                self.nodes.push(node);
                entry.insert(new_index);

                if let Some(tail_index) = self.tail.replace(new_index) {
                    if let Some(tail) = self.nodes.get_mut(tail_index) {
                        tail.next = Some(new_index);
                    } else {
                        log_or_panic!("Tail for signal dep list not found");
                    }
                }

                if self.head.is_none() {
                    self.head = Some(new_index);
                }
            }
            Entry::Occupied(entry) => {
                let Some(node) = self.nodes.get_mut(*entry.get()) else {
                    log_or_panic!("Slot hashmap entry out of bounds");
                    return;
                };

                // No need to move to end as there is no change
                // then this should already be in the correct relative position.
                if node.key.version == key.version {
                    return;
                }
                node.key.version = key.version;

                // if `next` is `None` we are tail.
                if let Some(next_index) = node.next.take() {
                    let previous_tail = self.tail.replace(*entry.get());
                    let previous = std::mem::replace(&mut node.previous, previous_tail);

                    match previous {
                        // We are head
                        None => {
                            let Some(next) = self.nodes.get_mut(next_index) else {
                                log_or_panic!("Next not found");
                                return;
                            };
                            next.previous = None;
                            self.head = Some(next_index);
                        }
                        // We are somewhere else
                        Some(previous_index) => {
                            let Some(next) = self.nodes.get_mut(next_index) else {
                                log_or_panic!("Next not found");
                                return;
                            };
                            next.previous = Some(previous_index);

                            let Some(previous) = self.nodes.get_mut(previous_index) else {
                                log_or_panic!("Previous not found");
                                return;
                            };
                            previous.next = Some(next_index);
                        }
                    }
                }
            }
        }
    }

    /// Create a iterator over the current nodes by moving them in.
    /// and clear the left over metadata.
    /// This re-uses that hashmap allocation.
    /// and allocates a new vec with the same capactiy (since its likely we will get close to the
    /// same amount of signals.)
    pub(crate) fn create_iter_and_clear(&mut self) -> IterSignalList {
        let new_vec = Vec::with_capacity(self.nodes.len());
        let iterator = IterSignalList {
            nodes: std::mem::replace(&mut self.nodes, new_vec),
            next: self.head,
        };
        self.items.clear();
        self.head = None;
        self.tail = None;
        iterator
    }
}

/// A iterator over the `HookKey`s in a `SignalDeoList`
pub(crate) struct IterSignalList {
    /// The linked list nodes
    nodes: Vec<SignalDepNode>,
    /// The index of the next node.
    next: Option<usize>,
}

impl Iterator for IterSignalList {
    type Item = HookKey;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.nodes.get(self.next?) {
            self.next = next.next;
            Some(next.key)
        } else {
            log_or_panic!("Next item not found in signal iterator.");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // No reason to have more insertion values than possible hooks
    static_assertions::assert_eq_size!(HookKey, InsertionOrder);
    // Ensure that the `SlotValue` enum does in fact use niche optimization on the `Box`;
    static_assertions::assert_eq_size!(SlotValue<()>, (InsertionOrder, Box<dyn ReactiveHook<()>>));
}
