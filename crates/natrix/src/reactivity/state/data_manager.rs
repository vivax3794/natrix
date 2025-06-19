//! Manger of the update cycle and signals.

use std::collections::BinaryHeap;

use super::{HookKey, State};
use crate::reactivity::component::Component;
use crate::reactivity::render_callbacks::UpdateResult;
use crate::reactivity::signal::SignalMethods;

/// Store some data but use `O` for its `Ord` implementation
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
    pub(crate) fn reg_dep(&mut self, dep: HookKey) {
        for signal in self.data.signals_mut() {
            signal.register_dep(dep);
        }
    }

    /// Loop over signals and update any depdant hooks for changed signals
    /// This also drains the deferred message queue
    // FIXME: Currently there is a memory leak if a signal is not modified for many cycles, while
    // also being part of a reactive hook which is triggering.
    // For example if have `|ctx: R<Self>| *ctx.foo + *ctx.bar`, and `foo` is modified 100 times
    // without `bar` being modified `bar`s dependency list will have 100 items in it.
    pub(crate) fn update(&mut self) {
        log::debug!("Performing update cycle for {}", std::any::type_name::<T>());
        self.drain_message_queue();

        let mut hooks = BinaryHeap::new();
        for signal in self.data.signals_mut() {
            if signal.changed() {
                for dep in signal.drain_dependencies() {
                    if let Some(dep_insertion_order) = self.hooks.insertion_order(dep) {
                        // PERF: This does not deduplicate the hooks.
                        hooks.push(OrderAssociatedData {
                            data: dep,
                            ordering: std::cmp::Reverse(dep_insertion_order),
                        });
                    }
                }
            }
        }

        log::trace!("{} hooks updating", hooks.len());
        while let Some(OrderAssociatedData { data: hook_key, .. }) = hooks.pop() {
            self.run_with_hook_and_self(hook_key, |ctx, hook| match hook.update(ctx, hook_key) {
                UpdateResult::Nothing => {}
                UpdateResult::RunHook(dep) => {
                    hooks.push(OrderAssociatedData {
                        data: dep,
                        ordering: std::cmp::Reverse(u64::MIN), // This item should be the next item
                    });
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
