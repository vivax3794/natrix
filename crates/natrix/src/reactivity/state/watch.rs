//! Implementation of `ctx.watch`

use super::{HookKey, RenderCtx};
use crate::Ctx;
use crate::error_handling::log_or_panic;
use crate::reactivity::render_callbacks::{ReactiveHook, UpdateResult};
use crate::reactivity::{State, statics};

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
    /// # #[derive(State)]
    /// # struct App {value: Signal<u32>}
    /// #
    /// # fn render() -> impl Element<App> {
    /// # |ctx: &mut RenderCtx<App>| {
    /// if ctx.watch(|ctx| *ctx.value > 2) {
    ///     e::div().text(|ctx: &mut RenderCtx<App>| *ctx.value)
    /// } else {
    ///     e::div().text("Value is too low")
    /// }
    /// # }}
    /// ```
    #[inline]
    pub fn watch<T, F>(&mut self, func: F) -> T
    where
        // TODO: Make this a owned lens
        F: Fn(&mut Ctx<C>) -> T + 'static,
        T: PartialEq + Clone + 'static,
    {
        let me = self.ctx.hooks.reserve_key();

        let result = self.ctx.track_reads(me, &func);

        let Some(dep) = statics::current_hook() else {
            log_or_panic!("`ctx.watch` called from outside a hook");
            return result;
        };
        let hook = WatchState {
            calc_value: Box::new(func),
            last_value: result.clone(),
            dep,
        };
        self.ctx.hooks.set_hook(me, Box::new(hook));
        self.render_state.hooks.push(me);

        result
    }
}
