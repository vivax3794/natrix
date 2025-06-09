//! Contains the non-reactive type

use crate::dom::attributes::AttributeResult;
use crate::dom::classes::ClassResult;
use crate::dom::element::{
    Element,
    ElementRenderResult,
    MaybeStaticElement,
    generate_fallback_node,
};
use crate::dom::{ToAttribute, ToClass};
use crate::error_handling::debug_panic;
use crate::reactivity::component::{Component, ComponentBase, NoMessages};
use crate::reactivity::signal::SignalMethods;
use crate::reactivity::state::ComponentData;

impl ComponentData for () {
    type FieldRef<'a> = [&'a mut dyn SignalMethods; 0];
    type SignalState = ();

    fn signals_mut(&mut self) -> Self::FieldRef<'_> {
        []
    }

    fn pop_signals(&mut self) -> Self::SignalState {}
    fn set_signals(&mut self, _state: Self::SignalState) {}
}

impl ComponentBase for () {
    type Data = ();

    fn into_data(self) -> Self::Data {}
}
impl Component for () {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;

    fn render() -> impl Element<Self> {
        debug_panic!(
            "Attempted to render a `()` as a component. This is most definitely not what you intended."
        );
        generate_fallback_node()
    }
}

/// Allows you to mark a node as non-reactive.
/// This should mainly be used when creating components that are generic over a type it wants to
/// render
///
/// As the naive solution:
/// ```compile_fail
/// # use natrix::prelude::*;
/// # #[derive(Component)]
/// # struct MyStruct<T>(T);
/// impl<T: Element<Self>> Component for MyStruct<T> {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> { e::div()}
/// # let _ = MyStruct(10);
/// }
/// ```
/// Causes a recursion loop in the trait analyzer, as it has to prove `Self: Component` to satisfy
/// `Element<Self>`, which again tries to satifiy `Element<Self>`.
///
/// The solution is to use `Element<()>` and `NonReactive`
///
/// ```rust
/// # use natrix::prelude::*;
/// # use natrix::reactivity::NonReactive;
/// #[derive(Component)]
/// struct MyStruct<T>(T);
///
/// impl<T: Element<()> + Copy> Component for MyStruct<T> {
///     # type EmitMessage = NoMessages;
///     # type ReceiveMessage = NoMessages;
///     fn render() -> impl Element<Self> {
///         e::div().child(|ctx: R<Self>| NonReactive(*ctx.0))
///     }
/// }
/// ```
pub struct NonReactive<E>(pub E);

impl<E: Element<()> + 'static, C: Component> Element<C> for NonReactive<E> {
    fn into_generic(self) -> MaybeStaticElement<C> {
        match self.0.into_generic() {
            MaybeStaticElement::Static(node) => MaybeStaticElement::Static(node),
            MaybeStaticElement::Html(html) => {
                if !html.deferred.is_empty() {
                    debug_panic!("Html element with reactive values in `NonReactive` context.");
                }
                MaybeStaticElement::Static(ElementRenderResult::Node(html.element.into()))
            }
            MaybeStaticElement::Dynamic(_) => {
                debug_panic!("Dynamic element in NonReactive context");
                MaybeStaticElement::Static(ElementRenderResult::Node(generate_fallback_node()))
            }
        }
    }
}

impl<A: ToAttribute<()>, C: Component> ToAttribute<C> for NonReactive<A> {
    type AttributeKind = A::AttributeKind;

    fn calc_attribute(self, name: &'static str, node: &web_sys::Element) -> AttributeResult<C> {
        match self.0.calc_attribute(name, node) {
            AttributeResult::SetIt(res) => AttributeResult::SetIt(res),
            AttributeResult::IsDynamic(_) => {
                debug_panic!("Dynamic Attribute in `NonReactive` context");
                AttributeResult::SetIt(None)
            }
        }
    }
}

impl<A: ToClass<()>, C: Component> ToClass<C> for NonReactive<A> {
    fn calc_class(self, node: &web_sys::Element) -> ClassResult<C> {
        match self.0.calc_class(node) {
            ClassResult::SetIt(res) => ClassResult::SetIt(res),
            ClassResult::Dynamic(_) => {
                debug_panic!("Dynamic Class in `NonReactive` context");
                ClassResult::SetIt(None)
            }
        }
    }
}
