//! Implementation of html elements, as well as helper constructors.
//!
//! This module is generally used via its alias in the prelude, `e`.
//! Most commonly you will just use the element functions directly, but you can construct a
//! `HtmlElement` instance if needed.
//!
//! # Example
//! ```ignore
//! # use natrix::prelude::*;
//! # let _: e::HtmlElement<(), _> =
//! e::div()
//!     .child(e::button().text("Click me"))
//!     .child(e::h1().text("Wow!"))
//! # ;
//! ```

use smallvec::SmallVec;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, intern};

use super::attributes::AttributeResult;
use super::classes::ClassResult;
use crate::css::selectors::{CompoundSelector, IntoCompoundSelector, SimpleSelector};
use crate::dom::element::{Element, MaybeStaticElement, generate_fallback_node};
use crate::dom::events::{Event, EventHandler};
use crate::dom::{ToAttribute, ToClass};
use crate::get_document;
use crate::reactivity::component::Component;
use crate::reactivity::signal::RenderingState;
use crate::reactivity::state::{EventToken, State};
use crate::utils::{debug_expect, debug_panic};

/// A deferred function to do something once state is available
pub(crate) type DeferredFunc<C> = Box<dyn FnOnce(&mut State<C>, &mut RenderingState)>;

/// A Generic html node with a given name.
#[must_use = "Web elements are useless if not rendered"]
pub struct HtmlElement<C: Component, T = ()> {
    /// The name of the tag
    pub(crate) element: web_sys::Element,
    /// The deferred actions
    pub(crate) deferred: SmallVec<[DeferredFunc<C>; 10]>,
    /// Phantom data to allow for genericity
    pub(crate) phantom: std::marker::PhantomData<T>,
}

impl<C: Component, T> HtmlElement<C, T> {
    /// Create a new html element with the specific tag
    ///
    /// All non-deprecated html elements have a helper function in this module
    pub fn new(tag: &'static str) -> Self {
        let node = if let Ok(node) = get_document().create_element(tag) {
            node
        } else {
            debug_panic!("Failed to create <{tag}>");
            generate_fallback_node().unchecked_into()
        };

        Self {
            element: node,
            deferred: SmallVec::new(),
            phantom: std::marker::PhantomData,
        }
    }

    /// Replace the tag type marker with `()` to allow returning different types of elements
    pub fn generic(self) -> HtmlElement<C, ()> {
        HtmlElement {
            element: self.element,
            deferred: self.deferred,
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
    /// e::button().on::<events::Click>(|ctx: E<Self>, _, _| {
    ///     *ctx.some_value += 1;
    /// })
    /// # }}
    /// ```
    /// For more information see [Reactivity](https://vivax3794.github.io/natrix/reactivity.html) in the book.
    #[inline]
    pub fn on<E: Event>(mut self, function: impl EventHandler<C, E>) -> Self {
        let function = function.func();
        let element = self.element.clone();

        self.deferred.push(Box::new(move |ctx, rendering_state| {
            let ctx_weak = ctx.deferred_borrow(EventToken::new());

            let callback: Box<dyn Fn(web_sys::Event) + 'static> = Box::new(move |event| {
                crate::return_if_panic!();

                let Ok(event) = event.dyn_into() else {
                    debug_panic!("Unexpected event type");
                    return;
                };

                let Some(mut ctx) = ctx_weak.borrow_mut() else {
                    debug_panic!("Component dropped without event handlers being cleaned up");
                    return;
                };

                ctx.clear();
                function(&mut ctx, EventToken::new(), event);
                ctx.update();
            });
            let closure = Closure::wrap(callback);
            let function = closure.as_ref().unchecked_ref();

            debug_expect!(
                element.add_event_listener_with_callback(intern(E::EVENT_NAME), function),
                "Failed to attach event handler"
            );

            rendering_state.keep_alive.push(Box::new(closure));
        }));
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
    #[inline]
    pub fn child<E: Element<C> + 'static>(mut self, child: E) -> Self {
        let node = match child.into_generic() {
            MaybeStaticElement::Static(result) => result.into_node(),
            MaybeStaticElement::Html(html) => {
                self.deferred.extend(html.deferred);
                html.element.into()
            }
            MaybeStaticElement::Dynamic(dynamic) => {
                let Ok(comment) = web_sys::Comment::new() else {
                    debug_panic!("Failed to create placeholder comment node");
                    return self;
                };
                let comment_clone = comment.clone();
                self.deferred.push(Box::new(move |ctx, rendering_state| {
                    let node = dynamic.render(ctx, rendering_state).into_node();
                    debug_expect!(
                        comment_clone.replace_with_with_node_1(&node),
                        "Failed to swap in child"
                    );
                }));
                comment.into()
            }
        };

        debug_expect!(self.element.append_child(&node), "Failed to append child");

        self
    }

    /// This is a simple alias for `child`
    #[inline]
    pub fn text<E: Element<C> + 'static>(self, text: E) -> Self {
        self.child(text)
    }

    /// Add a attribute to the node.
    #[inline]
    pub fn attr(mut self, key: &'static str, value: impl ToAttribute<C>) -> Self {
        match value.calc_attribute(key, &self.element) {
            AttributeResult::SetIt(res) => {
                if let Some(res) = res {
                    debug_expect!(
                        self.element.set_attribute(key, &res),
                        "Failed to set attribute"
                    );
                }
            }
            AttributeResult::IsDynamic(dynamic) => {
                self.deferred.push(Box::new(dynamic));
            }
        }

        self
    }

    /// Add a class to the element.
    #[inline]
    pub fn class(mut self, class: impl ToClass<C> + 'static) -> Self {
        match class.calc_class(&self.element) {
            ClassResult::SetIt(res) => {
                if let Some(res) = res {
                    debug_expect!(self.element.class_list().add_1(&res), "Failed to add class");
                }
            }
            ClassResult::Dynamic(dynamic) => {
                self.deferred.push(Box::new(dynamic));
            }
        }

        self
    }
}

impl<C: Component, T: 'static> Element<C> for HtmlElement<C, T> {
    #[inline]
    fn into_generic(self) -> MaybeStaticElement<C> {
        MaybeStaticElement::Html(self.generic())
    }
}

/// Implement a factory function that returns a `HtmlElement` with a tag name equal to the
/// function.
macro_rules! elements {
    ($($name:ident),*) => {
        $(
            pastey::paste! {
                #[doc = "<https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/" $name ">"]
                pub struct [< Tag $name:camel >];

                impl IntoCompoundSelector for [< Tag $name:camel >] {
                    fn into_compound(self) -> CompoundSelector {
                        CompoundSelector(vec![SimpleSelector::Tag(stringify!($name).into())])
                    }
                }

                #[doc = "<https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/" $name ">"]
                #[inline]
                pub fn $name<C: Component>() -> HtmlElement<C, [< Tag $name:camel >]> {
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
            pastey::paste! {
                impl<C: Component> HtmlElement<C, [< Tag $tag:camel >]> {
                    $(
                        #[doc = "<https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/" $tag "##" $attr ">"]
                        #[inline]
                        pub fn $attr(self, value: impl ToAttribute<C>) -> Self {
                            self.attr(stringify!($attr), value)
                           }
                    )+
                }
            }
        )*
    };
}

/// Generate a `attr` helpers implementation for the global attributes
macro_rules! global_attrs {
    ($($attr:ident),*) => {
        impl<C: Component, T> HtmlElement<C, T> {
            pastey::paste! {
                $(
                    #[doc = "<https://developer.mozilla.org/docs/Web/HTML/Reference/Global_attributes/" $attr ">"]
                    #[inline]
                    pub fn $attr(self, value: impl ToAttribute<C>) -> Self {
                        self.attr(stringify!($attr), value)
                    }
                )*
            }
        }
    };
}

/// Generate a `attr` helpers implementation for the aria attributes
macro_rules! aria_attrs {
    ($($attr:ident),*) => {
        impl<C: Component, T> HtmlElement<C, T> {
            $(
            pastey::paste! {
                #[doc = "<https://developer.mozilla.org/docs/Web/Accessibility/ARIA/Reference/Attributes/aria%2d" $attr ">"]
                #[inline]
                pub fn [<aria_$attr>](self, value: impl ToAttribute<C>) -> Self {
                    self.attr(concat!("aria-", stringify!($attr)), value)
                }
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
aria_attrs! {
autocomplete,
checked,
disabled,
errormessage,
expanded,
haspopup,
hidden,
invalid,
label,
level,
modal,
multiline,
multiselectable,
orientation,
placeholder,
pressed,
readonly,
required,
selected,
sort,
valuemax,
valuemin,
valuenow,
valuetext
}
