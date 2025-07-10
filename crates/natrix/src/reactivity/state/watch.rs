//! Implementation of `ctx.watch`
#![cfg(false)]

use super::{HookKey, RenderCtx};
use crate::Ctx;
use crate::reactivity::State;
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
    C: State,
    T: PartialEq,
    F: Fn(&mut Ctx<C>) -> T,
{
    fn update(&mut self, ctx: &mut Ctx<C>, you: HookKey) -> UpdateResult {
        let new_value = ctx.track_reads(you, &self.calc_value);

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

impl<C: State> RenderCtx<'_, C> {
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
        F: Fn(&mut Ctx<C>) -> T + 'static,
        T: PartialEq + Clone + 'static,
    {
        self.ctx.with_restore_signals(|ctx| {
            let me = ctx.hooks.reserve_key();

            let result = ctx.track_reads(me, &func);

            let hook = WatchState {
                calc_value: Box::new(func),
                last_value: result.clone(),
                dep: self.render_state.parent_dep,
            };
            ctx.hooks.set_hook(me, Box::new(hook));
            self.render_state.hooks.push(me);

            result
        })
    }
}
