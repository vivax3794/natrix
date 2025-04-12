//! Implementation of html elements, as well as helper constructors.
//!
//! This module is generally used via its alias in the prelude, `e`.
//! Most commonly you will just use the element functions directly, but you can construct a
//! `HtmlElement` instance if needed.
//!
//! # Example
//! ```rust
//! # use natrix::prelude::*;
//! # let _: e::HtmlElement<(), _> =
//! e::div()
//!     .child(e::button().text("Click me"))
//!     .child(e::h1().text("Wow!"))
//! # ;
//! ```

use std::borrow::Cow;
use std::rc::Weak;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, intern};

use crate::callbacks::EventHandler;
use crate::component::Component;
use crate::element::{Element, generate_fallback_node};
use crate::events::Event;
use crate::signal::RenderingState;
use crate::state::State;
use crate::utils::debug_expect;
use crate::{get_document, type_macros};

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
    ($type:ty) => {
        impl<C: Component> ToAttribute<C> for $type {
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
        impl<C: Component> ToAttribute<C> for $T {
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

/// A Generic html node with a given name.
#[must_use = "Web elements are useless if not rendered"]
pub struct HtmlElement<C: Component, T = ()> {
    /// The name of the tag
    tag: &'static str,
    /// List of child elements
    children: Vec<Box<dyn Element<C>>>,
    /// Events to be registered on the element
    events: Vec<(&'static str, Box<dyn Fn(&mut State<C>, web_sys::Event)>)>,
    /// Potentially dynamic attributes to apply
    attributes: Vec<(&'static str, Box<dyn ToAttribute<C>>)>,
    /// Css classes to apply
    classes: Vec<Cow<'static, str>>,
    /// Phantom data to allow for genericity
    phantom: std::marker::PhantomData<T>,
}

impl<C: Component, T> HtmlElement<C, T> {
    /// Create a new html element with the specific tag
    ///
    /// All non-deprecated html elements have a helper function in this module
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            events: Vec::new(),
            children: Vec::new(),
            attributes: Vec::new(),
            classes: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }

    /// Register a event handler for this element.
    ///
    /// The event handler is a closure taking a mutable reference to `S<Self>`.
    /// ```rust
    /// # use natrix::prelude::*;
    /// # #[derive(Component)]
    /// # struct MyComponent {
    /// #     some_value: i32,
    /// # }
    /// # impl Component for MyComponent {
    /// # type EmitMessage = NoMessages;
    /// # type ReceiveMessage = NoMessages;
    /// # fn render() -> impl Element<Self> {
    /// e::button().on::<events::Click>(|ctx: E<Self>, _| {
    ///     *ctx.some_value += 1;
    /// })
    /// # }}
    /// ```
    /// For more information see [Reactivity](TODO) in the book.
    pub fn on<E: Event>(mut self, function: impl EventHandler<C, E>) -> Self {
        let function = function.func();
        self.events.push((
            E::EVENT_NAME,
            Box::new(move |ctx, event| {
                if let Ok(event) = event.dyn_into::<E::JsEvent>() {
                    function(ctx, event);
                } else {
                    debug_assert!(false, "Mismatched event types");
                }
            }),
        ));
        self
    }

    /// Push a child to this element.
    /// This accepts any valid element including closures.
    /// ```rust
    /// # use natrix::prelude::*;
    /// # #[derive(Component)]
    /// # struct MyComponent {
    /// #     toggle: bool,
    /// # }
    /// # impl Component for MyComponent {
    /// # type EmitMessage = NoMessages;
    /// # type ReceiveMessage = NoMessages;
    /// # fn render() -> impl Element<Self> {
    /// e::div()
    ///     .child(e::h1().text("Wow!"))
    ///     .child(|ctx: R<Self>| {
    ///         if *ctx.toggle {
    ///             "Hello"
    ///         } else {
    ///             "World"
    ///         }
    ///     })
    /// # }}
    /// ```
    pub fn child<E: Element<C> + 'static>(mut self, child: E) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// This is a simple alias for `child`
    pub fn text<E: Element<C>>(self, text: E) -> Self {
        self.child(text)
    }

    /// Add a attribute to the node.
    ///
    /// See [Html Nodes](TODO) in the book for the schematics of the various valid attribute types.
    pub fn attr(mut self, key: &'static str, value: impl ToAttribute<C>) -> Self {
        self.attributes.push((key, Box::new(value)));
        self
    }

    /// Add a class to the element.
    pub fn class(mut self, class: impl Into<Cow<'static, str>>) -> Self {
        self.classes.push(class.into());
        self
    }
}

impl<C: Component, T: 'static> Element<C> for HtmlElement<C, T> {
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let Self {
            tag: name,
            events,
            children,
            attributes,
            classes,
            phantom: _,
        } = *self;

        let document = get_document();
        let Ok(element) = document.create_element(intern(name)) else {
            debug_assert!(false, "Failed to create element {name}");
            return generate_fallback_node();
        };

        for child in children {
            let child = child.render_box(ctx, render_state);
            debug_expect!(element.append_child(&child), "Failed to append child");
        }

        let ctx_weak = ctx.weak();
        for (event, function) in events {
            create_event_handler(&element, event, function, ctx_weak.clone(), render_state);
        }

        for (key, value) in attributes {
            value.apply_attribute(intern(key), &element, ctx, render_state);
        }
        for class in classes {
            debug_expect!(
                element.class_list().add_1(&class),
                "Failed to add class {class}"
            );
        }

        element.into()
    }
}

/// Wrap the given function in the needed reactivity machinery and set it as the event handler for
/// the specified event
fn create_event_handler<C: Component>(
    element: &web_sys::Element,
    event: &str,
    function: Box<dyn Fn(&mut State<C>, web_sys::Event)>,
    ctx_weak: Weak<std::cell::RefCell<State<C>>>,
    render_state: &mut RenderingState<'_>,
) {
    let callback: Box<dyn Fn(web_sys::Event) + 'static> = Box::new(move |event| {
        crate::return_if_panic!();

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
            paste::paste! {
                #[doc(hidden)]
                #[expect(non_camel_case_types, reason="We dont want to bother pulling in a case fold")]
                pub struct [< _$name >];

                #[doc = concat!("`<", stringify!($name), ">`")]
                pub fn $name<C: Component>() -> HtmlElement<C, [< _$name >]> {
                    HtmlElement::new(stringify!($name))
                }
            }
        )*
    };
}

/// A macro to define `attr` helpers for the the various elements
macro_rules! attr_helpers {
    ($($tag:ident => $($attr:ident),+;)*) => {
        $(
            paste::paste! {
                impl<C: Component> HtmlElement<C, [< _$tag >]> {
                    $(
                        #[doc = concat!("Set the `", stringify!($attr), "` attribute")]
                        pub fn $attr(self, value: impl ToAttribute<C>) -> Self {
                            self.attr(stringify!($attr), value)
                        }
                    )+
                }
            }
        )*
    };
}

/// Generate a `ToAttribute` implementation for the global attributes
macro_rules! global_attrs {
    ($($attr:ident),*) => {
        impl<C: Component, T> HtmlElement<C, T> {
            $(
                #[doc = concat!("Set the `", stringify!($attr), "` attribute")]
                pub fn $attr(self, value: impl ToAttribute<C>) -> Self {
                    self.attr(stringify!($attr), value)
                }
            )*
        }
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

attr_helpers! {
    a => href, target, rel, download, hreflang, referrerpolicy;
    audio => autoplay, controls, muted, preload, src;
    button => disabled, form, formaction, formenctype, formmethod, formnovalidate, formtarget, name, value;
    canvas => height, width;
    col => span;
    colgroup => span;
    details => open;
    embed => height, src, width;
    fieldset => disabled, form;
    form => acceptcharset, action, autocomplete, enctype, method, name, novalidate, target;
    iframe => allow, allowfullscreen, allowpaymentrequest, height, loading, name, referrerpolicy, sandbox, src, width;
    img => alt, crossorigin, decoding, height, ismap, loading, referrerpolicy, sizes, src, srcset, usemap, width;
    input => accept, alt, autocomplete, checked, dirname, disabled, form, formaction, formenctype, formmethod, formnovalidate, formtarget, height, list, max, maxlength, min, minlength, multiple, name, pattern, placeholder, readonly, required, size, src, step, value;
    li => value;
    map => name;
    meter => form, high, low, max, min, optimum, value;
    object => data, form, height, name, usemap, width;
    ol => reversed, start;
    optgroup => disabled, label;
    option => disabled, label, selected, value;
    picture => srcset;
    progress => max, value;
    script => crossorigin, defer, integrity, nomodule, referrerpolicy, src;
    select => autocomplete, disabled, form, multiple, name, required, size;
    source => media, sizes, src, srcset;
    summary => open;
    table => summary;
    textarea => autocomplete, cols, dirname, disabled, form, maxlength, minlength, name, placeholder, readonly, required, rows, wrap;
    time => datetime;
    track => default, kind, label, src, srclang;
    video => autoplay, controls, crossorigin, height, muted, playsinline, poster, preload, src, width;
}

// https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes
global_attrs! {
    autocapitalize, autofocus, enterkeyhint, inert, inputmode, nonce, role, writingsuggestions,
    accesskey, contenteditable, contextmenu, dir, draggable, dropzone, hidden, id, lang, spellcheck, style, tabindex, title, translate
}
