//! Implementation of the `Element` trait for various abstract types.

use std::borrow::Cow;

use crate::signal::RenderingState;
use crate::state::{ComponentData, State};

/// An `Element` is anything that can produce a DOM node.
/// The most common examples include `HtmlElement` and types like `String`.
///
/// Additionally, closures with the appropriate signature also implement this trait.
/// See the [Reactivity](TODO) chapter in the book for more examples.
///
/// 🚨 **You should generally NOT implement this trait manually.**
/// Instead, prefer **sub-components** (for stateful elements) or **stateless components**
/// (which are simply functions returning an `Element`).
///
/// ## ❌ Don't
/// Avoid manually implementing `Element` for custom components:
///
/// ```rust
/// struct MyFancyButton(&'static str);
/// impl<C> Element<C> for MyFancyButton {/* ... */}
/// ```
///
/// ## ✅ Do
/// Instead, use a **function-based stateless component**:
///
/// ```rust
/// fn my_fancy_button<C>(name: &'static str) -> impl Element<C> {
///     e::button() /* ... */
/// }
/// ```
///
/// This keeps your UI **cleaner, more composable, and easier to maintain**. 🚀✨
pub trait Element<C>: 'static {
    /// The actual implementation of the rendering.
    /// This is boxed to allow use as `dyn Element` for storing child nodes.
    #[doc(hidden)]
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node;

    /// A utility wrapper around `render_box` for when you have a concrete type.
    #[doc(hidden)]
    fn render(self, ctx: &mut State<C>, render_state: &mut RenderingState) -> web_sys::Node
    where
        Self: Sized,
    {
        Box::new(self).render_box(ctx, render_state)
    }
}

impl<C> Element<C> for web_sys::Node {
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) -> web_sys::Node {
        *self
    }
}

/// A simple Dom comment, used as a placeholder and replacement target.
pub struct Comment;

impl<C> Element<C> for Comment {
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) -> web_sys::Node {
        web_sys::Comment::new()
            .expect("Failed to make comment")
            .into()
    }
}

#[cfg(feature = "element_unit")]
impl<C: ComponentData> Element<C> for () {
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        Element::<C>::render(Comment, ctx, render_state)
    }
}

impl<T: Element<C>, C: ComponentData> Element<C> for Option<T> {
    #[inline]
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        match *self {
            Some(element) => element.render(ctx, render_state),
            None => Element::<C>::render(Comment, ctx, render_state),
        }
    }
}

impl<T: Element<C>, E: Element<C>, C: ComponentData> Element<C> for Result<T, E> {
    #[inline]
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        match *self {
            Ok(element) => element.render(ctx, render_state),
            Err(element) => element.render(ctx, render_state),
        }
    }
}

impl<C> Element<C> for &'static str {
    #[inline]
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let text = web_sys::Text::new().expect("Failed to make text");
        text.set_text_content(Some(*self));
        text.into()
    }
}

impl<C> Element<C> for String {
    #[inline]
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let text = web_sys::Text::new().expect("Failed to make text");
        text.set_text_content(Some(&self));
        text.into()
    }
}

impl<C> Element<C> for Cow<'static, str> {
    #[inline]
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let text = web_sys::Text::new().expect("Failed to make text");
        text.set_text_content(Some(&self));
        text.into()
    }
}

/// Generate a implemention of `Element` for a specific integer type.
///
/// This uses the `itoa` crate for fast string conversions.
///
/// Note: The reason we can not do a blanket implemention on `itoa::Integer` here is that it would
/// conflict with the blanket closure implementation of `Element` (Thanks rust :/)
macro_rules! int_element {
    ($T:ident) => {
        impl<C> Element<C> for $T {
            fn render_box(
                self: Box<Self>,
                _ctx: &mut State<C>,
                _render_state: &mut RenderingState,
            ) -> web_sys::Node {
                let mut buffer = itoa::Buffer::new();
                let result = buffer.format(*self);

                let text = web_sys::Text::new().expect("Failed to make text");
                text.set_text_content(Some(result));
                text.into()
            }
        }
    };
}

/// Call the `int_element!` macro for all the the specified int types
macro_rules! int_elements {
    ($($T:ident),*) => {
        $(int_element!{$T})*
    };
}

int_elements! {u8, u16, u32, u64, u128, i8, i16, i32, i128, usize, isize }
