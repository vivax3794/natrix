//! Manger of the update cycle and signals.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use slotmap::SecondaryMap;

use super::{HookKey, State};
use crate::error_handling::log_or_panic;
use crate::reactivity::component::Component;
use crate::reactivity::render_callbacks::UpdateResult;
use crate::reactivity::signal::SignalMethods;

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
    vectors: Vec<std::vec::IntoIter<HookKey>>,
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
    iter: &mut std::vec::IntoIter<HookKey>,
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
        mut vectors: Vec<std::vec::IntoIter<HookKey>>,
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

            // NOTE: We know the `source_index` is valid, but I dont think there is a nice way in rust
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

/// Trait automatically implemented on reactive structs by the derive macro.
///
/// This trait provides the internal interface between component data structures
/// and the reactive system. It allows the framework to:
/// - Access all reactive signals within a component
/// - Capture the current state of all signals
/// - Restore signals to a previously captured state
///
/// This is an internal trait not meant for direct implementation by users.
#[doc(hidden)]
pub trait ComponentData: Sized + 'static {
    /// References to all reactive signals in this component.
    ///
    /// This is typically implemented as an array of mutable references to the component's
    /// signal fields, allowing the reactive system to track and update them.
    type FieldRef<'a>: IntoIterator<Item = &'a mut dyn SignalMethods>;

    /// A complete snapshot of all signal values in this component.
    ///
    /// This type captures the entire signal state for later restoration,
    /// typically used for nested reactive contexts such as `.watch`.
    type SignalState;

    /// Returns mutable references to all signals in this component.
    ///
    /// This allows the reactive system to track modifications and trigger
    /// updates when signal values change.
    fn signals_mut(&mut self) -> Self::FieldRef<'_>;

    /// Extracts the current signal state and resets signals to their default state.
    fn pop_signals(&mut self) -> Self::SignalState;

    /// Restores all signals to a previously captured state.
    fn set_signals(&mut self, state: Self::SignalState);
}

impl<T: Component> State<T> {
    /// Clear all signals
    pub(crate) fn clear(&mut self) {
        for signal in self.data.signals_mut() {
            signal.clear();
        }
    }

    /// Register a dependency for all read signals
    /// INVARIANT: Hooks must call `.reg_dep` in the relative order they are required to be updated and invalidated.
    pub(crate) fn reg_dep(&mut self, dep: HookKey) {
        for signal in self.data.signals_mut() {
            signal.register_dep(dep);
        }
    }

    /// Loop over signals and update any depdant hooks for changed signals
    /// This also drains the deferred message queue
    /// INVARIANT: `.clear` and `.update` should always come in pairs around user code, and you
    /// should never yield control to js between them, as other state mutation code will (should)
    /// call `.clear` in the middle of your operation if you do.
    pub(crate) fn update(&mut self) {
        self.drain_message_queue();
        log::trace!("Performing update cycle for {}", std::any::type_name::<T>());

        let dep_lists: Vec<_> = self
            .data
            .signals_mut()
            .into_iter()
            .filter(|signal| signal.changed())
            .map(|signal| signal.drain_dependencies().into_iter())
            .collect();

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
}
