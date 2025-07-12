//! Implementation of `ctx.watch`

use super::{HookKey, RenderCtx};
use crate::error_handling::log_or_panic;
use crate::reactivity::render_callbacks::{ReactiveHook, RenderingState, UpdateResult};
use crate::reactivity::state::InnerCtx;
use crate::reactivity::{KeepAlive, State, statics};

/// The wather hook / signal
struct WatchState<F, T> {
    /// Function to calculate the state
    calc_value: F,
    /// The previous cached value
    last_value: T,
    /// The dependency that owns us.
    dep: HookKey,
    /// Keepalive
    keep_alive: Vec<KeepAlive>,
    /// Child hooks
    hooks: Vec<HookKey>,
}

impl<C, F, T> ReactiveHook<C> for WatchState<F, T>
where
    C: State,
    T: PartialEq,
    F: Fn(&mut RenderCtx<C>) -> T,
{
    fn update(&mut self, ctx: &mut InnerCtx<C>, you: HookKey) -> UpdateResult {
        self.keep_alive.clear();
        let hooks = std::mem::take(&mut self.hooks);

        let new_value = ctx.track_reads(you, |ctx| {
            let mut render = RenderCtx {
                ctx,
                render_state: RenderingState {
                    keep_alive: &mut self.keep_alive,
                    hooks: &mut self.hooks,
                },
            };
            (self.calc_value)(&mut render)
        });

        if new_value == self.last_value {
            UpdateResult::DropHooks(hooks)
        } else {
            UpdateResult::RunHook(self.dep, hooks)
        }
    }

    fn drop_us(self: Box<Self>) -> Vec<HookKey> {
        Vec::new()
    }
}

impl<C: State> RenderCtx<'_, '_, C> {
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
        F: for<'c, 's> Fn(&mut RenderCtx<'c, 's, C>) -> T + 'static,
        T: PartialEq + Clone + 'static,
    {
        let me = self.ctx.hooks.reserve_key();
        let mut hooks = Vec::new();
        let mut keep_alive = Vec::new();

        let keep_alive_borrow = &mut keep_alive;
        let hooks_borrow = &mut hooks;

        let result = self.ctx.track_reads(me, |ctx| {
            let mut render = RenderCtx {
                ctx,
                render_state: RenderingState {
                    keep_alive: keep_alive_borrow,
                    hooks: hooks_borrow,
                },
            };
            func(&mut render)
        });

        let Some(dep) = statics::current_hook() else {
            log_or_panic!("`ctx.watch` called from outside a hook");
            return result;
        };
        let hook = WatchState {
            calc_value: Box::new(func),
            last_value: result.clone(),
            dep,
            keep_alive,
            hooks,
        };
        self.ctx.hooks.set_hook(me, Box::new(hook));
        self.render_state.hooks.push(me);

        result
    }
}
