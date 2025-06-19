//! Implementation of `ctx.watch`

use super::data_manager::ComponentData;
use super::{HookKey, RenderCtx};
use crate::State;
use crate::reactivity::component::Component;
use crate::reactivity::render_callbacks::{ReactiveHook, UpdateResult};

/// The wather hook / signal
struct WatchState<F, T> {
    /// Function to calculate the state
    calc_value: F,
    /// The previous cached value
    last_value: T,
    /// The dependency that owns us.
    dep: HookKey,
}

impl<C, F, T> ReactiveHook<C> for WatchState<F, T>
where
    C: Component,
    T: PartialEq,
    F: Fn(&mut State<C>) -> T,
{
    fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
        ctx.clear();
        let new_value = (self.calc_value)(ctx);
        ctx.reg_dep(you);

        if new_value == self.last_value {
            UpdateResult::Nothing
        } else {
            UpdateResult::RunHook(self.dep)
        }
    }

    fn drop_us(self: Box<Self>) -> Vec<HookKey> {
        Vec::new()
    }
}

impl<C: Component> RenderCtx<'_, C> {
    /// Calculate the value using the function and cache it using `clone`.
    /// Then whenever any signals read in the function are modified re-run the function and check
    /// if the new result is different.
    /// Only reruns the caller when the item is different.
    ///
    /// # Example
    /// ```rust
    /// # use natrix::prelude::*;
    /// # #[derive(Component)]
    /// # struct MyComponent {value: u32}
    /// #
    /// # impl Component for MyComponent {
    /// # type EmitMessage = NoMessages;
    /// # type ReceiveMessage = NoMessages;
    /// # fn render() -> impl Element<Self> {
    /// # |ctx: R<Self>| {
    /// if ctx.watch(|ctx| *ctx.value > 2) {
    ///     e::div().text(|ctx: R<Self>| *ctx.value)
    /// } else {
    ///     e::div().text("Value is too low")
    /// }
    /// # }}}
    /// ```
    #[inline]
    pub fn watch<T, F>(&mut self, func: F) -> T
    where
        F: Fn(&State<C>) -> T + 'static,
        T: PartialEq + Clone + 'static,
    {
        self.watch_mut(move |ctx| func(ctx))
    }

    /// Internal only version of mutable version of watch
    #[doc(hidden)]
    pub fn watch_mut<T, F>(&mut self, func: F) -> T
    where
        F: Fn(&mut State<C>) -> T + 'static,
        T: PartialEq + Clone + 'static,
    {
        let signal_state = self.ctx.data.pop_signals();

        let result = func(self.ctx);

        let hook = WatchState {
            calc_value: Box::new(func),
            last_value: result.clone(),
            dep: self.render_state.parent_dep,
        };
        let me = self.ctx.hooks.insert_hook(Box::new(hook));
        self.ctx.reg_dep(me);
        self.render_state.hooks.push(me);

        self.ctx.data.set_signals(signal_state);

        result
    }
}
