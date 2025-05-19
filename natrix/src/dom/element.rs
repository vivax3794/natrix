//! Implementation of the `Element` trait for various abstract types.

use super::HtmlElement;
use crate::reactivity::component::Component;
use crate::reactivity::render_callbacks::ReactiveNode;
use crate::reactivity::signal::RenderingState;
use crate::reactivity::state::{RenderCtx, State};
use crate::type_macros;
use crate::utils::debug_panic;

/// A result of the rendering process.
pub(crate) enum ElementRenderResult {
    /// A generic node.
    Node(web_sys::Node),
    /// A text node.
    Text(Box<str>),
}

impl ElementRenderResult {
    /// Convert to a `web_sys::Node`.
    pub(crate) fn into_node(self) -> web_sys::Node {
        match self {
            ElementRenderResult::Node(node) => node,
            ElementRenderResult::Text(text) => {
                if let Ok(node) = web_sys::Text::new_with_data(&text) {
                    node.into()
                } else {
                    debug_panic!("Failed to create text node");
                    generate_fallback_node()
                }
            }
        }
    }
}

/// A element that doesnt depend on any state
pub(crate) trait StaticElement {
    /// Render the element.
    fn render(self) -> ElementRenderResult;
}

/// A element that might or might not depend on state.
pub enum MaybeStaticElement<C: Component> {
    /// A already statically rendered element.
    Static(ElementRenderResult),
    /// A html element
    Html(HtmlElement<C, ()>),
    /// A element that needs to be rendered.
    Dynamic(Box<dyn DynElement<C>>),
}

impl<C: Component> MaybeStaticElement<C> {
    /// Convert the element into a `web_sys::Node`.
    pub(crate) fn render(
        self,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> ElementRenderResult {
        match self {
            MaybeStaticElement::Static(element) => element,
            MaybeStaticElement::Html(html) => {
                let HtmlElement {
                    element,
                    deferred,
                    phantom: _,
                } = html;

                for modification in deferred {
                    modification(ctx, render_state);
                }

                ElementRenderResult::Node(element.into())
            }
            MaybeStaticElement::Dynamic(element) => element.render(ctx, render_state),
        }
    }
}

/// Convert a element into a `MaybeStaticElement`.
pub trait Element<C: Component>: 'static {
    /// Convert the element into a `MaybeStaticElement`.
    fn into_generic(self) -> MaybeStaticElement<C>;
}

/// A dynamic element
pub(crate) trait DynElement<C: Component> {
    /// Render the element.
    fn render(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> ElementRenderResult;
}

impl StaticElement for web_sys::Node {
    #[inline]
    fn render(self) -> ElementRenderResult {
        ElementRenderResult::Node(self)
    }
}

impl<T: StaticElement> StaticElement for Option<T> {
    #[inline]
    fn render(self) -> ElementRenderResult {
        match self {
            Some(element) => element.render(),
            None => generate_fallback_node().render(),
        }
    }
}

impl<T: StaticElement, E: StaticElement> StaticElement for Result<T, E> {
    #[inline]
    fn render(self) -> ElementRenderResult {
        match self {
            Ok(element) => element.render(),
            Err(element) => element.render(),
        }
    }
}

/// Generate a Element implementation for a type that can be converted to `&str`
macro_rules! string_element {
    ($t:ty) => {
        impl StaticElement for $t {
            fn render(self) -> ElementRenderResult {
                ElementRenderResult::Text((*self).to_string().into_boxed_str())
            }
        }
    };
}
type_macros::strings!(string_element);

/// Generate a implementation of `Element` for a specific integer type.
macro_rules! int_element {
    ($T:ident, $fmt:ident) => {
        impl StaticElement for $T {
            fn render(self) -> ElementRenderResult {
                let mut buffer = $fmt::Buffer::new();
                let result = buffer.format(self);

                ElementRenderResult::Text(result.to_string().into_boxed_str())
            }
        }
    };
}
type_macros::numerics!(int_element);

/// Attempt to create a comment node.
/// If this fails (wrongly) convert the error to a comment node.
/// This allows us to satisfy a non-Result `web_sys::Node` return type.
/// This conversion should never happen, but if it does, code down the line will simply hit a error
/// and will ignore it as needed.
pub(crate) fn generate_fallback_node() -> web_sys::Node {
    web_sys::Comment::new()
        .unwrap_or_else(wasm_bindgen::JsCast::unchecked_into)
        .into()
}

/// Impl `Element` for a static element
macro_rules! impl_to_static {
    ($t:ty $(, $fmt:ident)?) => {
        impl<C: Component> Element<C> for $t {
            #[inline]
            fn into_generic(self) -> MaybeStaticElement<C> {
                MaybeStaticElement::Static(self.render())
            }
        }
    };
}

impl_to_static!(web_sys::Node);
type_macros::strings!(impl_to_static);
type_macros::numerics!(impl_to_static);

impl<T: Element<C> + 'static, C: Component> Element<C> for Option<T> {
    #[inline]
    fn into_generic(self) -> MaybeStaticElement<C> {
        match self {
            Some(element) => element.into_generic(),
            None => MaybeStaticElement::Static(generate_fallback_node().render()),
        }
    }
}

impl<T: Element<C> + 'static, E: Element<C> + 'static, C: Component> Element<C> for Result<T, E> {
    #[inline]
    fn into_generic(self) -> MaybeStaticElement<C> {
        match self {
            Ok(element) => element.into_generic(),
            Err(element) => element.into_generic(),
        }
    }
}

impl<C: Component> Element<C> for MaybeStaticElement<C> {
    #[inline]
    fn into_generic(self) -> MaybeStaticElement<C> {
        self
    }
}

impl<F, C, R> DynElement<C> for F
where
    F: Fn(&mut RenderCtx<C>) -> R + 'static,
    R: Element<C> + 'static,
    C: Component,
{
    fn render(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> ElementRenderResult {
        let this = *self;
        let (me, node) =
            ReactiveNode::create_initial(Box::new(move |ctx| this(ctx).into_generic()), ctx);
        render_state.hooks.push(me);
        ElementRenderResult::Node(node)
    }
}

impl<F, C, R> Element<C> for F
where
    F: Fn(&mut RenderCtx<C>) -> R + 'static,
    R: Element<C> + 'static,
    C: Component,
{
    fn into_generic(self) -> MaybeStaticElement<C> {
        MaybeStaticElement::Dynamic(Box::new(self))
    }
}
