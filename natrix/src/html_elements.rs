//! Implemention of html elements, as well as helper constructors.
//!
//! This module is generally used via its alias in the prelude, `e`.
//! Most commonly you will just use the element functions directly, but you can construct a
//! `HtmlElement` instance if needed.
//!
//! # Example
//! ```rust
//! e::div()
//!     .child(e::button().text("Click me"))
//!     .child(e::h1().text("Wow!"))
//! ```

use std::borrow::Cow;
use std::rc::Weak;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, intern};

use crate::callbacks::EventHandler;
use crate::element::Element;
use crate::events::Event;
use crate::signal::RenderingState;
use crate::state::{ComponentData, State};
use crate::utils::debug_expect;
use crate::{get_document, type_macros};

/// A trait for using a arbitrary type as a attribute value.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid attribute value.",
    note = "Try converting the value to a string"
)]
pub trait ToAttribute<C>: 'static {
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
    ($type:ty) => {
        impl<C> ToAttribute<C> for $type {
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
    ($T:ident, $fmt:ident) => {
        impl<C> ToAttribute<C> for $T {
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

impl<C> ToAttribute<C> for bool {
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

impl<C, T: ToAttribute<C>> ToAttribute<C> for Option<T> {
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
impl<C, T: ToAttribute<C>, E: ToAttribute<C>> ToAttribute<C> for Result<T, E> {
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

/// A Generic html node with a given name.
#[must_use = "Web elements are useless if not rendered"]
pub struct HtmlElement<C> {
    /// The name of the tag
    tag: &'static str,
    /// List of child elements
    children: Vec<Box<dyn Element<C>>>,
    /// Events to be registered on the element
    events: Vec<(&'static str, Box<dyn Fn(&mut State<C>, web_sys::Event)>)>,
    // We use `Vec` over `HashMap` here to avoid overhead because we are never doing lookups and
    // the js side is already doing deduplication.
    /// Inline css styles to apply
    styles: Vec<(&'static str, Cow<'static, str>)>,
    /// Potentially dynamic attributes to apply
    attributes: Vec<(&'static str, Box<dyn ToAttribute<C>>)>,
}

impl<C> HtmlElement<C> {
    /// Create a new html element with the specific tag
    ///
    /// All non-deprecated html elements have a helper function in this module
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            events: Vec::new(),
            children: Vec::new(),
            styles: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// Register a event handler for this element.
    ///
    /// The event handler is a closure taking a mutable reference to `S<Self>`.
    /// ```rust
    /// e::button().on("click", |ctx: &mut S<Self>| {
    ///     *ctx.some_value += 1;
    /// })
    /// ```
    /// For more information see [Reactivity](TODO) in the book.
    pub fn on<E: Event>(mut self, function: impl EventHandler<C, E::JsEvent>) -> Self {
        let function = function.func();
        self.events.push((
            E::EVENT_NAME,
            Box::new(move |ctx, event| {
                if let Ok(event) = event.dyn_into::<E::JsEvent>() {
                    function(ctx, event);
                } else {
                    debug_assert!(false, "Missmatched event types");
                }
            }),
        ));
        self
    }

    /// Push a child to this element.
    /// This accepts any valid element including closures.
    /// ```rust
    /// e::div()
    ///     .child(e::h1().text("Wow!"))
    ///     .child(|ctx: &S<Self>| {
    ///         if *ctx.toggle {
    ///             "Hello"
    ///         } else {
    ///             "World"
    ///         }
    ///     })
    /// ```
    pub fn child<E: Element<C> + 'static>(mut self, child: E) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// This is a simple alias for `child`
    pub fn text<E: Element<C>>(self, text: E) -> Self {
        self.child(text)
    }

    /// Adds a inline style to the element
    // (This isnt reactive because in the future we will suggest using proper static css as well as
    // provide reactive css vars)
    pub fn style(mut self, key: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        self.styles.push((key, value.into()));
        self
    }

    /// Add a attribute to the node.
    ///
    /// See [Html Nodes](TODO) in the book for the schematics of the various valid attribute types.
    pub fn attr(mut self, key: &'static str, value: impl ToAttribute<C>) -> Self {
        self.attributes.push((key, Box::new(value)));
        self
    }

    /// Shorthand for `.attr("id", ...)` with a specific ID.
    pub fn id(self, id: &'static str) -> Self {
        self.attr("id", id)
    }
}

impl<C: ComponentData> Element<C> for HtmlElement<C> {
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let Self {
            tag: name,
            events,
            children,
            styles,
            attributes,
        } = *self;

        let document = get_document();
        #[expect(
            clippy::panic,
            reason = "No good recovery for not being able to create a element"
        )]
        let element = document
            .create_element(intern(name))
            .unwrap_or_else(|_| panic!("Failed to create element {name}"));

        for child in children {
            let child = child.render_box(ctx, render_state);
            debug_expect!(element.append_child(&child), "Failed to append child");
        }

        let ctx_weak = ctx.weak();
        for (event, function) in events {
            create_event_handler(&element, event, function, ctx_weak.clone(), render_state);
        }

        #[expect(
            clippy::arithmetic_side_effects,
            reason = "This should only fail if we are out of memory, which basically all collections silently panic for"
        )]
        let style = styles
            .into_iter()
            .map(|(key, value)| key.to_owned() + ":" + &value + ";")
            .collect::<String>();
        debug_expect!(
            element.set_attribute(intern("style"), &style),
            "Failed to set style"
        );

        for (key, value) in attributes {
            value.apply_attribute(intern(key), &element, ctx, render_state);
        }

        element.into()
    }
}

/// Wrap the given function in the needed reactivity machinery and set it as the event handler for
/// the specified event
fn create_event_handler<C: ComponentData>(
    element: &web_sys::Element,
    event: &str,
    function: Box<dyn Fn(&mut State<C>, web_sys::Event)>,
    ctx_weak: Weak<std::cell::RefCell<State<C>>>,
    render_state: &mut RenderingState<'_>,
) {
    let callback: Box<dyn Fn(web_sys::Event) + 'static> = Box::new(move |event| {
        let Some(ctx) = ctx_weak.upgrade() else {
            debug_assert!(
                false,
                "Component dropped without event handlers being cleaned up"
            );
            return;
        };
        let mut ctx = ctx.borrow_mut();

        ctx.clear();
        function(&mut ctx, event);
        ctx.update();
    });
    let closure = Closure::wrap(callback);
    let function = closure.as_ref().unchecked_ref();

    debug_expect!(
        element.add_event_listener_with_callback(intern(event), function),
        "Failed to attach event handler"
    );

    render_state.keep_alive.push(Box::new(closure));
}

/// Implement a factory function that returns a `HtmlElement` with a tag name equal to the
/// function.
macro_rules! elements {
    ($($name:ident),*) => {
        $(
            // Note to self: Do not put every possible html tag inline in your docs
            #[doc = concat!("`<", stringify!($name), ">`")]
            #[inline(always)]
            pub fn $name<C>() -> HtmlElement<C> {
                HtmlElement::new(stringify!($name))
            }
        )*
    };
}

// https://developer.mozilla.org/en-US/docs/Web/HTML/Element
elements! {
h1, h2, h3, h4, h5, h6,
address, article, aside, footer, header, hgroup, main, nav, section, search,
blockquote, dd, div, dl, dt, figcaption, figure, hr, li, menu, ol, p, pre, ul,
a, abbr, b, bdi, bdo, br, cite, code, data, dfn, em, i, kbd, mark, q, rp, rt, ruby, s, samp, small, span, strong, sub, sup, time, u, var, wbr,
area, audio, img, map, track, video,
embed, fencedframe, iframe, object, picture, source,
svg, math,
canvas, script,
del, ins,
caption, col, colgroup, table, tbody, td, tfoot, th, thead, tr,
button, datalist, fieldset, form, input, label, legend, meter, optgroup, option, output, progress, select, textarea,
details, dialog, summary
}
