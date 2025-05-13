//! Implementation of the `Element` trait for various abstract types.

use crate::component::Component;
use crate::signal::RenderingState;
use crate::state::State;
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

/// An `Element` is anything that can produce a DOM node.
/// The most common examples include `HtmlElement` and types like `String`.
///
/// Additionally, closures with the appropriate signature also implement this trait.
/// See the [Reactivity](https://vivax3794.github.io/natrix/reactivity.html) chapter in the book for more examples.
///
/// 🚨 **You should generally NOT implement this trait manually.**
/// Instead, prefer **sub-components** (for stateful elements) or **stateless components**
/// (which are simply functions returning an `Element`).
///
/// ## ❌ Don't
/// Avoid manually implementing `Element` for custom components:
///
/// ```ignore
/// struct MyFancyButton(&'static str);
/// impl<C: Component> Element<C> for MyFancyButton {/* ... */}
/// ```
///
/// ## ✅ Do
/// Instead, use a **function-based stateless component**:
///
/// ```rust
/// # use natrix::prelude::*;
/// fn my_fancy_button<C: Component>(name: &'static str) -> impl Element<C> {
///     e::button().text(name)
/// }
/// ```
///
/// This keeps your UI **cleaner, more composable, and easier to maintain**. 🚀✨
pub trait Element<C: Component>: 'static {
    /// The actual implementation of the rendering.
    /// This is boxed to allow use as `dyn Element` for storing child nodes.
    #[doc(hidden)]
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> ElementRenderResult;

    /// A utility wrapper around `render_box` for when you have a concrete type.
    #[doc(hidden)]
    #[inline]
    fn render(self, ctx: &mut State<C>, render_state: &mut RenderingState) -> ElementRenderResult
    where
        Self: Sized,
    {
        Box::new(self).render_box(ctx, render_state)
    }

    /// Wrap this element in a `Box`.
    /// This lets you easially return different element types from the same function.
    #[inline]
    fn into_box(self) -> Box<dyn Element<C>>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl<C: Component> Element<C> for Box<dyn Element<C>> {
    #[inline]
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> ElementRenderResult {
        (*self).render_box(ctx, render_state)
    }

    #[inline]
    fn render(self, ctx: &mut State<C>, render_state: &mut RenderingState) -> ElementRenderResult
    where
        Self: Sized,
    {
        self.render_box(ctx, render_state)
    }

    #[inline]
    fn into_box(self) -> Box<dyn Element<C>>
    where
        Self: Sized,
    {
        self
    }
}

impl<C: Component> Element<C> for web_sys::Node {
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) -> ElementRenderResult {
        ElementRenderResult::Node(*self)
    }
}

impl<T: Element<C>, C: Component> Element<C> for Option<T> {
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> ElementRenderResult {
        match *self {
            Some(element) => element.render(ctx, render_state),
            None => ElementRenderResult::Node(generate_fallback_node()),
        }
    }
}

impl<T: Element<C>, E: Element<C>, C: Component> Element<C> for Result<T, E> {
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> ElementRenderResult {
        match *self {
            Ok(element) => element.render(ctx, render_state),
            Err(element) => element.render(ctx, render_state),
        }
    }
}

/// Generate a Element implementation for a type that can be converted to `&str`
macro_rules! string_element {
    ($t:ty) => {
        impl<C: Component> Element<C> for $t {
            fn render_box(
                self: Box<Self>,
                _ctx: &mut State<C>,
                _render_state: &mut RenderingState,
            ) -> ElementRenderResult {
                ElementRenderResult::Text((*self).to_string().into_boxed_str())
            }
        }
    };
}

type_macros::strings!(string_element);

/// Generate a implementation of `Element` for a specific integer type.
///
/// This uses the `itoa` crate for fast string conversions.
///
/// Note: The reason we can not do a blanket implementation on `itoa::Integer` here is that it would
/// conflict with the blanket closure implementation of `Element` (Thanks rust :/)
macro_rules! int_element {
    ($T:ident, $fmt:ident) => {
        impl<C: Component> Element<C> for $T {
            fn render_box(
                self: Box<Self>,
                _ctx: &mut State<C>,
                _render_state: &mut RenderingState,
            ) -> ElementRenderResult {
                let mut buffer = $fmt::Buffer::new();
                let result = buffer.format(*self);

                ElementRenderResult::Text(result.to_string().into_boxed_str())
            }
        }
    };
}

type_macros::numerics!(int_element);

#[cfg(feature = "either")]
/// Impl of `Element` on `Either`
mod either_element {
    use either::Either;

    use super::{Component, Element, ElementRenderResult, RenderingState, State};

    impl<A: Element<C>, B: Element<C>, C: Component> Element<C> for Either<A, B> {
        fn render_box(
            self: Box<Self>,
            ctx: &mut State<C>,
            render_state: &mut RenderingState,
        ) -> ElementRenderResult {
            match *self {
                Either::Left(a) => a.render(ctx, render_state),
                Either::Right(b) => b.render(ctx, render_state),
            }
        }
    }
}

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
