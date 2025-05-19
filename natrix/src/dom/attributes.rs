//! Convert various values to html attributes

use std::borrow::Cow;

use super::html_elements::DeferredFunc;
use crate::reactivity::component::Component;
use crate::reactivity::render_callbacks::{ReactiveAttribute, SimpleReactive};
use crate::reactivity::state::RenderCtx;
use crate::type_macros;

/// The result of apply attribute
pub(crate) enum AttributeResult<C: Component> {
    /// The attribute was set
    SetIt(Option<Cow<'static, str>>),
    /// The attribute was dynamic
    IsDynamic(DeferredFunc<C>),
}

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
    fn calc_attribute(self, name: &'static str, node: &web_sys::Element) -> AttributeResult<C>;
}

/// generate a `ToAttribute` implementation for a string type
macro_rules! attribute_string {
    ($t:ty, $cow:expr) => {
        impl<C: Component> ToAttribute<C> for $t {
            fn calc_attribute(
                self,
                _name: &'static str,
                _node: &web_sys::Element,
            ) -> AttributeResult<C> {
                AttributeResult::SetIt(Some(($cow)(self)))
            }
        }
    };
}

type_macros::strings_cow!(attribute_string);

/// generate `ToAttribute` for a int using itoa
macro_rules! attribute_int {
    ($t:ident, $fmt:ident) => {
        impl<C: Component> ToAttribute<C> for $t {
            fn calc_attribute(
                self,
                _name: &'static str,
                _node: &web_sys::Element,
            ) -> AttributeResult<C> {
                let mut buffer = $fmt::Buffer::new();
                let result = buffer.format(self);

                AttributeResult::SetIt(Some(Cow::from(result.to_string())))
            }
        }
    };
}

type_macros::numerics!(attribute_int);

impl<C: Component> ToAttribute<C> for bool {
    fn calc_attribute(self, _name: &'static str, _node: &web_sys::Element) -> AttributeResult<C> {
        AttributeResult::SetIt(self.then(|| Cow::from("")))
    }
}

impl<C: Component, T: ToAttribute<C>> ToAttribute<C> for Option<T> {
    fn calc_attribute(self, name: &'static str, node: &web_sys::Element) -> AttributeResult<C> {
        if let Some(inner) = self {
            inner.calc_attribute(name, node)
        } else {
            AttributeResult::SetIt(None)
        }
    }
}

impl<C: Component, T: ToAttribute<C>, E: ToAttribute<C>> ToAttribute<C> for Result<T, E> {
    fn calc_attribute(self, name: &'static str, node: &web_sys::Element) -> AttributeResult<C> {
        match self {
            Ok(inner) => inner.calc_attribute(name, node),
            Err(inner) => inner.calc_attribute(name, node),
        }
    }
}

impl<F, C, R> ToAttribute<C> for F
where
    F: Fn(&mut RenderCtx<C>) -> R + 'static,
    R: ToAttribute<C>,
    C: Component,
{
    fn calc_attribute(self, name: &'static str, node: &web_sys::Element) -> AttributeResult<C> {
        let node = node.clone();

        AttributeResult::IsDynamic(Box::new(move |ctx, render_state| {
            let hook = SimpleReactive::init_new(
                Box::new(move |ctx| ReactiveAttribute {
                    name,
                    data: self(ctx),
                }),
                node.clone(),
                ctx,
            );
            render_state.hooks.push(hook);
        }))
    }
}
