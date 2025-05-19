//! Apply and update html classes

use std::borrow::Cow;

use super::html_elements::DeferredFunc;
use crate::reactivity::Component;
use crate::reactivity::render_callbacks::{ReactiveClass, SimpleReactive};
use crate::reactivity::state::RenderCtx;
use crate::type_macros;

/// The result of applying a class
pub(crate) enum ClassResult<C: Component> {
    /// The class was applied immedtialy, along with its value
    AppliedIt(Option<Cow<'static, str>>),
    /// The class needs access to state
    Dynamic(DeferredFunc<C>),
}

/// A trait for converting a value to a class name
pub trait ToClass<C: Component> {
    /// Convert the value to a class name
    fn calc_class(self, node: &web_sys::Element) -> ClassResult<C>;
}

/// Generate a `ToClass` implementation for a string type
macro_rules! class_string {
    ($type:ty, $cow:expr) => {
        impl<C: Component> ToClass<C> for $type {
            fn calc_class(self, _node: &web_sys::Element) -> ClassResult<C> {
                ClassResult::AppliedIt(Some(($cow)(self)))
            }
        }
    };
}
type_macros::strings_cow!(class_string);

impl<C: Component, T: ToClass<C>> ToClass<C> for Option<T> {
    fn calc_class(self, node: &web_sys::Element) -> ClassResult<C> {
        if let Some(inner) = self {
            inner.calc_class(node)
        } else {
            ClassResult::AppliedIt(None)
        }
    }
}

impl<C: Component, T: ToClass<C>, E: ToClass<C>> ToClass<C> for Result<T, E> {
    fn calc_class(self, node: &web_sys::Element) -> ClassResult<C> {
        match self {
            Ok(inner) => inner.calc_class(node),
            Err(inner) => inner.calc_class(node),
        }
    }
}

impl<F, C, R> ToClass<C> for F
where
    F: Fn(&mut RenderCtx<C>) -> R + 'static,
    R: ToClass<C> + 'static,
    C: Component,
{
    fn calc_class(self, node: &web_sys::Element) -> ClassResult<C> {
        let node = node.clone();
        ClassResult::Dynamic(Box::new(move |ctx, rendering_state| {
            let hook = SimpleReactive::init_new(
                Box::new(move |ctx| ReactiveClass { data: self(ctx) }),
                node.clone(),
                ctx,
            );
            rendering_state.hooks.push(hook);
        }))
    }
}
