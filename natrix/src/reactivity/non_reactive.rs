//! Contains the non-reactive type

use crate::dom::attributes::AttributeResult;
use crate::dom::classes::ClassResult;
use crate::dom::element::{DynElement, Element, ElementRenderResult, MaybeStaticElement};
use crate::dom::{ToAttribute, ToClass, ToCssValue};
use crate::reactivity::component::{Component, ComponentBase, NoMessages};
use crate::reactivity::signal::{RenderingState, SignalMethods};
use crate::reactivity::state::{ComponentData, State};
use crate::utils::debug_panic;

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
        crate::dom::element::generate_fallback_node()
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
        MaybeStaticElement::Dynamic(Box::new(self))
    }
}

impl<E: Element<()>, C: Component> DynElement<C> for NonReactive<E> {
    fn render(
        self: Box<Self>,
        _ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> ElementRenderResult {
        self.0
            .into_generic()
            .render(&mut State::create_base(()), render_state)
    }
}

impl<A: ToAttribute<()>, C: Component> ToAttribute<C> for NonReactive<A> {
    fn apply_attribute(self, name: &'static str, node: &web_sys::Element) -> AttributeResult<C> {
        self.0.apply_attribute(name, node);
        AttributeResult::SetIt
    }
}

impl<A: ToClass<()>, C: Component> ToClass<C> for NonReactive<A> {
    fn apply_class(self, node: &web_sys::Element) -> ClassResult<C> {
        match self.0.apply_class(node) {
            ClassResult::AppliedIt(res) => ClassResult::AppliedIt(res),
            ClassResult::Dynamic(_) => {
                debug_panic!("Dynamic class in `NonReactive` context");
                ClassResult::AppliedIt(None)
            }
        }
    }
}

impl<Css: ToCssValue<()>, C: Component> ToCssValue<C> for NonReactive<Css> {
    fn apply_css(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::HtmlElement,
        _ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) {
        Box::new(self.0).apply_css(name, node, &mut State::create_base(()), render_state);
    }
}
