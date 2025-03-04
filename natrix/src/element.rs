//! Implementation of the `Element` trait for various abstract types.

use std::borrow::Cow;

use crate::signal::RenderingState;
use crate::state::{ComponentData, State};

/// The actual implementation of a `Element`.
/// This is sealed because we need user to be able to refer to `Element`, but we do not want them
/// to be able to implement it as a incorrect `Element` implementation can lead to runtime panics.
/// And all usecases *should* be covered by components (both statefull and stateless)
pub(crate) trait SealedElement<C>: 'static {
    /// The actual implementation of the rendering.
    /// This is boxed to allow use as `dyn Element` for storing child nodes.
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node;

    /// A utility wrapper around `render_box` for when you have a concrete type.
    #[inline(always)]
    fn render(self, ctx: &mut State<C>, render_state: &mut RenderingState) -> web_sys::Node
    where
        Self: Sized,
    {
        Box::new(self).render_box(ctx, render_state)
    }
}

/// A `Element` is anything that can produce a dom node, this is most commonly `HtmlElement`, as
/// well as types such as `String`.
///
/// In addition closures (with the approperiate) signature implement this trait, see the
/// [Reactivity](TODO) chapther in the book for examples.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a element.",
    label = "Expected valid element"
)]
pub trait Element<C>: SealedElement<C> {}
impl<C, T: SealedElement<C>> Element<C> for T {}

impl<C> SealedElement<C> for web_sys::Node {
    #[inline(always)]
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

impl<C> SealedElement<C> for Comment {
    #[inline(always)]
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
impl<C: ComponentData> SealedElement<C> for () {
    #[inline]
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        SealedElement::<C>::render(Comment, ctx, render_state)
    }
}

impl<T: SealedElement<C>, C: ComponentData> SealedElement<C> for Option<T> {
    #[inline]
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        match *self {
            Some(element) => element.render(ctx, render_state),
            None => SealedElement::<C>::render(Comment, ctx, render_state),
        }
    }
}

impl<T: SealedElement<C>, E: SealedElement<C>, C: ComponentData> SealedElement<C> for Result<T, E> {
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

impl<C> SealedElement<C> for &'static str {
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

impl<C> SealedElement<C> for String {
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

impl<C> SealedElement<C> for Cow<'static, str> {
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
        impl<C> SealedElement<C> for $T {
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
