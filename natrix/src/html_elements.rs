use std::borrow::Cow;
use std::rc::Weak;

use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, intern};

use crate::callbacks::Event;
use crate::element::SealedElement;
use crate::get_document;
use crate::prelude::debug;
use crate::signal::RenderingState;
use crate::state::{ComponentData, State};

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid attribute value.",
    note = "Try converting the value to a string"
)]
pub(crate) trait ToAttribute<C>: 'static {
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        ctx: &mut State<C>,
        rendering_state: &mut RenderingState,
    );
}

impl<C> ToAttribute<C> for &'static str {
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        _ctx: &mut State<C>,
        _rendering_state: &mut RenderingState,
    ) {
        node.set_attribute(name, *self).unwrap();
    }
}

impl<C> ToAttribute<C> for String {
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        _ctx: &mut State<C>,
        _rendering_state: &mut RenderingState,
    ) {
        node.set_attribute(name, &self).unwrap();
    }
}

impl<C> ToAttribute<C> for Cow<'static, str> {
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        _ctx: &mut State<C>,
        _rendering_state: &mut RenderingState,
    ) {
        node.set_attribute(name, &self).unwrap();
    }
}

impl<C> ToAttribute<C> for bool {
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        _ctx: &mut State<C>,
        _rendering_state: &mut RenderingState,
    ) {
        if *self {
            node.set_attribute(name, "").unwrap();
        } else {
            node.remove_attribute(name).unwrap();
        }
    }
}

#[must_use = "Web elements are useless if not rendered"]
pub struct HtmlElement<C> {
    name: &'static str,
    events: Vec<(&'static str, Box<dyn Fn(&mut State<C>)>)>,
    children: Vec<Box<dyn SealedElement<C>>>,
    styles: Vec<(&'static str, Cow<'static, str>)>,
    attributes: Vec<(&'static str, Box<dyn ToAttribute<C>>)>,
}

impl<C> HtmlElement<C> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            events: Vec::new(),
            children: Vec::new(),
            styles: Vec::new(),
            attributes: Vec::new(),
        }
    }

    pub fn on(mut self, event: &'static str, function: impl Event<C>) -> Self {
        self.events.push((event, function.func()));
        self
    }

    pub fn child<E: SealedElement<C> + 'static>(mut self, child: E) -> Self {
        self.children.push(Box::new(child));
        self
    }

    pub fn text<E: SealedElement<C>>(self, text: E) -> Self {
        self.child(text)
    }

    pub fn style(mut self, key: &'static str, value: impl Into<Cow<'static, str>>) -> Self {
        self.styles.push((key, value.into()));
        self
    }

    pub fn attr(mut self, key: &'static str, value: impl ToAttribute<C>) -> Self {
        self.attributes.push((key, Box::new(value)));
        self
    }

    pub fn id(self, id: &'static str) -> Self {
        self.attr("id", id)
    }
}

impl<C: ComponentData> SealedElement<C> for HtmlElement<C> {
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let Self {
            name,
            events,
            children,
            styles,
            attributes,
        } = *self;

        let document = get_document();
        let element = document
            .create_element(intern(name))
            .expect("Failed to get document");

        for child in children {
            let child = child.render_box(ctx, render_state);
            element
                .append_child(&child)
                .expect("Failed to append child");
        }

        let ctx_weak = ctx.weak();
        for (event, function) in events {
            let new_ctx = Weak::clone(&ctx_weak);
            let callback: Box<dyn Fn() + 'static> = Box::new(move || {
                debug("Running Event Handler");
                let data = new_ctx
                    .upgrade()
                    .expect("Component dropped in event callback");

                let mut data = data.borrow_mut();

                data.clear();
                function(&mut data);
                data.update();
            });

            let closure = Closure::<dyn Fn()>::wrap(callback);
            let function = closure.as_ref().unchecked_ref();
            element
                .add_event_listener_with_callback(intern(event), function)
                .expect("Failed to add listener");

            render_state.keep_alive.push(Box::new(closure));
        }

        let style = styles
            .into_iter()
            .map(|(key, value)| key.to_owned() + ":" + &value + ";")
            .collect::<Vec<_>>()
            .join("");
        element
            .set_attribute(intern("style"), &style)
            .expect("Failed to set style");

        for (key, value) in attributes {
            value.apply_attribute(intern(key), &element, ctx, render_state);
        }

        element.into()
    }
}

macro_rules! elements {
    ($($name:ident),*) => {
        $(
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
