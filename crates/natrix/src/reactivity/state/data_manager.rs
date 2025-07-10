//! Manger of the update cycle and signals.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use slotmap::SecondaryMap;

use super::{Ctx, HookKey};
use crate::error_handling::log_or_panic;
use crate::reactivity::render_callbacks::UpdateResult;

/// Store some data but use `O` for its `Ord` implementation
#[derive(Debug)]
struct OrderAssociatedData<T, O> {
    /// The data in question
    data: T,
    /// The value to order based on
    ordering: O,
}

impl<T, O: PartialEq> PartialEq for OrderAssociatedData<T, O> {
    fn eq(&self, other: &Self) -> bool {
        self.ordering == other.ordering
    }
}
impl<T, O: Eq> Eq for OrderAssociatedData<T, O> {}

impl<T, O: PartialOrd> PartialOrd for OrderAssociatedData<T, O> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ordering.partial_cmp(&other.ordering)
    }
}
impl<T, O: Ord> Ord for OrderAssociatedData<T, O> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordering.cmp(&other.ordering)
    }
}

/// The queue processor of the queue
struct HookQueue {
    /// All changed vectors
    vectors: Vec<indexmap::set::IntoIter<HookKey>>,
    /// The queue of the next item in each vector
    queue: BinaryHeap<OrderAssociatedData<(HookKey, usize), Reverse<u64>>>,
    /// A temporary next item
    next_item: Option<HookKey>,
    /// Last processed hook to avoid duplicates
    last_hook: Option<HookKey>,
}

/// Iterate over `iter` until it finds a key in `insertion_orders`
fn get_next_valid(
    insertion_orders: &SecondaryMap<HookKey, u64>,
    iter: &mut indexmap::set::IntoIter<HookKey>,
) -> Option<(HookKey, u64)> {
    for hook in iter.by_ref() {
        if let Some(&order) = insertion_orders.get(hook) {
            return Some((hook, order));
        }
    }
    None
}

impl HookQueue {
    /// Create a new queue
    fn new(
        insertion_orders: &SecondaryMap<HookKey, u64>,
        mut vectors: Vec<indexmap::set::IntoIter<HookKey>>,
    ) -> Self {
        let mut queue = BinaryHeap::with_capacity(vectors.len());

        let first_items = vectors
            .iter_mut()
            .enumerate()
            .filter_map(|(index, vector)| {
                let (hook, ordering) = get_next_valid(insertion_orders, vector)?;
                Some(OrderAssociatedData {
                    data: (hook, index),
                    ordering: Reverse(ordering),
                })
            });
        queue.extend(first_items);

        Self {
            vectors,
            queue,
            next_item: None,
            last_hook: None,
        }
    }

    /// Push a item to be popped next
    fn push_next(&mut self, key: HookKey) {
        if self.next_item.is_some() {
            log_or_panic!("`push_next` called while `next_item` already has item");
        }

        self.next_item = Some(key);
    }

    /// Pop the next item
    fn pop(&mut self, insertion_orders: &SecondaryMap<HookKey, u64>) -> Option<HookKey> {
        if let Some(next) = self.next_item.take() {
            self.last_hook = Some(next);
            return Some(next);
        }

        loop {
            log::trace!("current queue: {:?}", self.queue);
            let (hook, source_index) = self.queue.pop()?.data;

            // PERF: We know the `source_index` is valid, but I dont think there is a nice way in rust
            // to enforce that invariant (maybe something fancy with marker lifetimes)
            // Hopefully rust optimizes out the bounds check?
            if let Some(vector) = self.vectors.get_mut(source_index) {
                while let Some((next_hook, ordering)) = get_next_valid(insertion_orders, vector) {
                    if Some(next_hook) != self.last_hook {
                        self.queue.push(OrderAssociatedData {
                            data: (next_hook, source_index),
                            ordering: Reverse(ordering),
                        });
                        break;
                    }
                }
            } else {
                log_or_panic!(
                    "`source_index` {source_index} out of range of HookQueue vectors list (len {})",
                    self.vectors.len()
                );
            }

            // Skip duplicates
            if Some(hook) == self.last_hook {
                continue;
            }

            self.last_hook = Some(hook);
            return Some(hook);
        }
    }
}

// PERF: the Full loop over signals to check which changed is a concern.
// In theory we could have signals hold a `Weak<RefCell<Vec<...>>>`, but having per-signal
// Borrows is exactly what we want to avoid, and would defeat a lot of performance benefits we
// have.
// We could use a threading-queue? I wonder if there are any single-threaded mpsc queues.

/// Trait automatically implemented on reactive structs by the derive macro.
#[doc(hidden)]
pub trait State: Sized + 'static {
    /// If you were read register this as a dependency
    /// Also should clear your read flag
    #[doc(hidden)]
    fn reg_dep(&mut self, dep: HookKey);
    /// Return a Vector of dependency lists from changed sources.
    /// Should also clear both flags.
    fn dirty_deps_lists(&mut self) -> impl Iterator<Item = indexmap::set::IntoIter<HookKey>>;
    // TODO: Method to set the state value with a `Self`.
}

impl State for () {
    fn reg_dep(&mut self, _dep: HookKey) {}
    fn dirty_deps_lists(&mut self) -> impl Iterator<Item = indexmap::set::IntoIter<HookKey>> {
        std::iter::empty()
    }
}

impl<T: State> Ctx<T> {
    /// Loop over signals and update any depdant hooks for changed signals
    /// This also drains the deferred message queue
    fn update(&mut self) {
        log::trace!("Performing update cycle for {}", std::any::type_name::<T>());

        let dep_lists: Vec<_> = self.data.dirty_deps_lists().collect();

        log::trace!("{} signals changed", dep_lists.len());
        let mut hook_queue = HookQueue::new(&self.hooks.insertion_order, dep_lists);

        while let Some(hook_key) = hook_queue.pop(&self.hooks.insertion_order) {
            log::trace!("Updating hook {hook_key:?}");
            self.run_with_hook_and_self(hook_key, |ctx, hook| match hook.update(ctx, hook_key) {
                UpdateResult::Nothing => {}
                UpdateResult::RunHook(dep) => {
                    hook_queue.push_next(dep);
                }
                UpdateResult::DropHooks(deps) => {
                    for dep in deps {
                        ctx.hooks.drop_hook(dep);
                    }
                }
            });
        }
        log::trace!("Update cycle complete");
    }

    /// Run the given method and track the reactive modifications done in it.
    /// and call initiate the update cycle afterwards.
    pub(crate) fn track_changes<R>(&mut self, func: impl FnOnce(&mut Self) -> R) -> R {
        let result = func(self);
        self.update();
        result
    }

    /// Run the given method and track reads, registering the given hook as a dependency of read
    /// signals
    pub(crate) fn track_reads<R>(
        &mut self,
        hook: HookKey,
        func: impl for<'a> FnOnce(&'a mut Self) -> R,
    ) -> R {
        let result = func(self);
        self.reg_dep(hook);
        result
    }

    /// Run the given function pop and restoring signals around it
    // PERF: This causes 2 loops of all signals, and then the end of the reactive scope is a extra
    // one.
    // I.e using `ctx.watch` makes the hook 4X more expensive to compute. (Pop, Check, Set, Check)
    #[cfg(false)]
    pub(crate) fn with_restore_signals<R>(
        &mut self,
        func: impl for<'a> FnOnce(&'a mut Self) -> R,
    ) -> R {
        let state = self.data.pop_signals();
        let result = func(self);
        self.data.set_signals(state);
        result
    }
}
