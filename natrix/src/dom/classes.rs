//! Apply and update html classes

use std::borrow::Cow;

use crate::reactivity::render_callbacks::{ReactiveClass, SimpleReactive};
use crate::reactivity::signal::RenderingState;
use crate::reactivity::state::RenderCtx;
use crate::reactivity::{Component, State};
use crate::type_macros;
use crate::utils::debug_expect;

/// A trait for converting a value to a class name
pub trait ToClass<C: Component> {
    /// Convert the value to a class name
    fn apply_class(
        self: Box<Self>,
        node: &web_sys::Element,
        ctx: &mut State<C>,
        rendering_state: &mut RenderingState,
    ) -> Option<Cow<'static, str>>;
}

/// Generate a `ToClass` implementation for a string type
macro_rules! class_string {
    ($type:ty, $cow:expr) => {
        impl<C: Component> ToClass<C> for $type {
            fn apply_class(
                self: Box<Self>,
                node: &web_sys::Element,
                _ctx: &mut State<C>,
                _rendering_state: &mut RenderingState,
            ) -> Option<Cow<'static, str>> {
                let class_list = node.class_list();
                debug_expect!(class_list.add_1(&self), "Failed to add class {self}");
                Some(($cow)(*self))
            }
        }
    };
}
type_macros::strings_cow!(class_string);

impl<C: Component, T: ToClass<C>> ToClass<C> for Option<T> {
    fn apply_class(
        self: Box<Self>,
        node: &web_sys::Element,
        ctx: &mut State<C>,
        rendering_state: &mut RenderingState,
    ) -> Option<Cow<'static, str>> {
        if let Some(inner) = *self {
            Box::new(inner).apply_class(node, ctx, rendering_state)
        } else {
            None
        }
    }
}

impl<C: Component, T: ToClass<C>, E: ToClass<C>> ToClass<C> for Result<T, E> {
    fn apply_class(
        self: Box<Self>,
        node: &web_sys::Element,
        ctx: &mut State<C>,
        rendering_state: &mut RenderingState,
    ) -> Option<Cow<'static, str>> {
        match *self {
            Ok(inner) => Box::new(inner).apply_class(node, ctx, rendering_state),
            Err(inner) => Box::new(inner).apply_class(node, ctx, rendering_state),
        }
    }
}

impl<F, C, R> ToClass<C> for F
where
    F: Fn(&mut RenderCtx<C>) -> R + 'static,
    R: ToClass<C> + 'static,
    C: Component,
{
    fn apply_class(
        self: Box<Self>,
        node: &web_sys::Element,
        ctx: &mut State<C>,
        rendering_state: &mut RenderingState,
    ) -> Option<Cow<'static, str>> {
        let hook = SimpleReactive::init_new(
            Box::new(move |ctx| ReactiveClass { data: self(ctx) }),
            node.clone(),
            ctx,
        );
        rendering_state.hooks.push(hook);
        None
    }
}
