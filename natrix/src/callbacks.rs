//! Implementations of various traits for closures.

use std::borrow::Cow;

use crate::component::Component;
use crate::element::Element;
use crate::events::Event;
use crate::html_elements::{ToAttribute, ToClass, ToCssValue};
use crate::render_callbacks::{
    ReactiveAttribute,
    ReactiveClass,
    ReactiveCss,
    ReactiveNode,
    SimpleReactive,
};
use crate::signal::RenderingState;
use crate::state::{EventToken, RenderCtx, State};

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

impl<C, F, T> ToCssValue<C> for F
where
    C: Component,
    T: ToCssValue<C> + 'static,
    F: Fn(&mut RenderCtx<C>) -> T + 'static,
{
    fn apply_css(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::HtmlElement,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) {
        let hook = SimpleReactive::init_new(
            Box::new(move |ctx| ReactiveCss {
                property: name,
                data: self(ctx),
            }),
            node.clone().into(),
            ctx,
        );
        render_state.hooks.push(hook);
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
    fn func(self) -> impl Fn(&mut State<C>, EventToken, E::JsEvent) + 'static;
}
impl<C, E: Event, F: Fn(&mut State<C>, EventToken, E::JsEvent) + 'static> EventHandler<C, E> for F {
    fn func(self) -> impl Fn(&mut State<C>, EventToken, E::JsEvent) + 'static {
        self
    }
}
