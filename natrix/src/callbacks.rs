//! Implementations of various traits for closures.

use crate::element::Element;
use crate::html_elements::ToAttribute;
use crate::render_callbacks::{ReactiveAttribute, ReactiveNode, SimpleReactive};
use crate::signal::RenderingState;
use crate::state::{ComponentData, State};

impl<F, C, R> Element<C> for F
where
    F: Fn(&State<C>) -> R + 'static,
    R: Element<C> + 'static,
    C: ComponentData,
{
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let (hook, node) = ReactiveNode::create_inital(Box::new(self), ctx);

        render_state.keep_alive.push(Box::new(hook.0));
        node
    }
}

impl<F, C, R> ToAttribute<C> for F
where
    F: Fn(&State<C>) -> R + 'static,
    R: ToAttribute<C>,
    C: ComponentData,
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
        rendering_state.keep_alive.push(Box::new(hook));
    }
}

/// Utility trait for use in stateless components
///
/// When defining a stateless component it is much easier to use `impl Event<C>` than writting out
/// the whole function trait yourself.
///
/// ```
/// fn my_button<C>(click: impl Event<C>) -> impl Element<C> {
///     e::button().on("click", click)
/// }
/// ```
pub trait Event<C> {
    /// Return a boxed version of the function in this event
    fn func(self) -> Box<dyn Fn(&mut State<C>)>;
}
impl<C, F: Fn(&mut State<C>) + 'static> Event<C> for F {
    fn func(self) -> Box<dyn Fn(&mut State<C>)> {
        Box::new(self)
    }
}
