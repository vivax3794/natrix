//! Convert various values to html attributes

use crate::reactivity::State;
use crate::reactivity::component::Component;
use crate::reactivity::render_callbacks::{ReactiveAttribute, SimpleReactive};
use crate::reactivity::signal::RenderingState;
use crate::reactivity::state::RenderCtx;
use crate::type_macros;
use crate::utils::debug_expect;

/// A trait for using a arbitrary type as a attribute value.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid attribute value.",
    note = "Try converting the value to a string"
)]
pub trait ToAttribute<C: Component>: 'static {
    /// Modify the given node to have the attribute set
    ///
    /// We use this apply system instead of returning the value as some types will also need to
    /// conditionally remove the attribute
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        ctx: &mut State<C>,
        rendering_state: &mut RenderingState,
    );
}

/// generate a `ToAttribute` implementation for a string type
macro_rules! attribute_string {
    ($t:ty) => {
        impl<C: Component> ToAttribute<C> for $t {
            fn apply_attribute(
                self: Box<Self>,
                name: &'static str,
                node: &web_sys::Element,
                _ctx: &mut State<C>,
                _rendering_state: &mut RenderingState,
            ) {
                debug_expect!(
                    node.set_attribute(name, &self),
                    "Failed to set attribute {name}"
                );
            }
        }
    };
}

type_macros::strings!(attribute_string);

/// generate `ToAttribute` for a int using itoa
macro_rules! attribute_int {
    ($t:ident, $fmt:ident) => {
        impl<C: Component> ToAttribute<C> for $t {
            fn apply_attribute(
                self: Box<Self>,
                name: &'static str,
                node: &web_sys::Element,
                _ctx: &mut State<C>,
                _rendering_state: &mut RenderingState,
            ) {
                let mut buffer = $fmt::Buffer::new();
                let result = buffer.format(*self);

                debug_expect!(
                    node.set_attribute(name, result),
                    "Failed to set attribute {name}"
                );
            }
        }
    };
}

type_macros::numerics!(attribute_int);

impl<C: Component> ToAttribute<C> for bool {
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        _ctx: &mut State<C>,
        _rendering_state: &mut RenderingState,
    ) {
        if *self {
            debug_expect!(
                node.set_attribute(name, ""),
                "Failed to set attribute {name}"
            );
        } else {
            debug_expect!(
                node.remove_attribute(name),
                "Failed to remove attribute {name}"
            );
        }
    }
}

impl<C: Component, T: ToAttribute<C>> ToAttribute<C> for Option<T> {
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        ctx: &mut State<C>,
        rendering_state: &mut RenderingState,
    ) {
        if let Some(inner) = *self {
            Box::new(inner).apply_attribute(name, node, ctx, rendering_state);
        } else {
            debug_expect!(
                node.remove_attribute(name),
                "Failed to remove attribute {name}"
            );
        }
    }
}

impl<C: Component, T: ToAttribute<C>, E: ToAttribute<C>> ToAttribute<C> for Result<T, E> {
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        ctx: &mut State<C>,
        rendering_state: &mut RenderingState,
    ) {
        match *self {
            Ok(inner) => Box::new(inner).apply_attribute(name, node, ctx, rendering_state),
            Err(inner) => Box::new(inner).apply_attribute(name, node, ctx, rendering_state),
        }
    }
}

impl<F, C, R> ToAttribute<C> for F
where
    F: Fn(&mut RenderCtx<C>) -> R + 'static,
    R: ToAttribute<C>,
    C: Component,
{
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        ctx: &mut State<C>,
        rendering_state: &mut RenderingState,
    ) {
        let hook = SimpleReactive::init_new(
            Box::new(move |ctx| ReactiveAttribute {
                name,
                data: self(ctx),
            }),
            node.clone(),
            ctx,
        );
        rendering_state.hooks.push(hook);
    }
}
