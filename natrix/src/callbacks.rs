//! Implementations of various traits for closures.

use crate::component::Component;
use crate::element::Element;
use crate::events::Event;
use crate::html_elements::ToAttribute;
use crate::render_callbacks::{ReactiveAttribute, ReactiveNode, SimpleReactive};
use crate::signal::RenderingState;
use crate::state::{RenderCtx, State};

impl<F, C, R> Element<C> for F
where
    F: Fn(&mut RenderCtx<C>) -> R + 'static,
    R: Element<C> + 'static,
    C: Component,
{
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let (me, node) = ReactiveNode::create_initial(Box::new(self), ctx);
        render_state.hooks.push(me);
        node
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

/// Utility trait for use in stateless components
///
/// When defining a stateless component it is much easier to use `impl Event<C>` than writing out
/// the whole function trait yourself.
///
/// ```
/// # use natrix::prelude::*;
/// # use natrix::callbacks::EventHandler;
/// fn my_button<C: Component>(click: impl EventHandler<C, events::Click>) -> impl Element<C> {
///     e::button().on::<events::Click>(click)
/// }
/// ```
pub trait EventHandler<C, E: Event> {
    /// Return a boxed version of the function in this event
    fn func(self) -> impl Fn(&mut State<C>, E::JsEvent) + 'static;
}
impl<C, E: Event, F: Fn(&mut State<C>, E::JsEvent) + 'static> EventHandler<C, E> for F {
    fn func(self) -> impl Fn(&mut State<C>, E::JsEvent) + 'static {
        self
    }
}
