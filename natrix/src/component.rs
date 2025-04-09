//! Component traits

use std::cell::RefCell;
use std::rc::Rc;

use crate::element::Element;
use crate::get_document;
use crate::html_elements::ToAttribute;
use crate::signal::{RenderingState, SignalMethods};
use crate::state::{ComponentData, HookKey, S, State};
use crate::utils::SmallAny;

/// The base component, this is implemented by the `#[derive(Component)]` macro and handles
/// associating a component with its reactive state as well as converting to a struct to its
/// reactive counter part
#[diagnostic::on_unimplemented(
    message = "`{Self}` Missing `#[derive(Component)]`.",
    note = "`#[derive(Component)]` Required for implementing `Component`"
)]
pub trait ComponentBase: Sized + 'static {
    /// The reactive version of this struct.
    /// Should be used for most locations where a "Component" is expected.
    type Data: ComponentData;

    /// Convert this struct into its reactive variant.
    fn into_data(self) -> Self::Data;

    /// Convert this to its reactive variant and wrap it in the component state struct.
    fn into_state(self) -> Rc<RefCell<State<Self>>>
    where
        Self: Component,
    {
        State::new(self.into_data())
    }
}

/// A type that has no possible values.
/// Similar to the stdlib `!` type.
// The reason we do not use `std::convert::Infallible` is
// That we do not want the auto trait to be not implemented on say `Result<Self, !>`
// Why a component would declare a message type with `!` is beyond me, but it is possible
// (I suppose it could be the result of Generics)
pub enum NoMessages {}

/// Trait to allow us to deny calling message handlers on components that do not emit messages.
///
/// This is behind the feature flag instead of being automatic because it affects the
/// public API of the framework, even if the stuff it breaks is already likely to be a bug.
#[cfg(feature = "nightly")]
pub(crate) auto trait IsntNever {}
#[cfg(feature = "nightly")]
impl !IsntNever for NoMessages {}

/// Trait to allow us to deny calling message handlers on components that do not emit messages.
///
/// Always impleneted on stable
#[cfg(not(feature = "nightly"))]
pub(crate) trait IsntNever {}
#[cfg(not(feature = "nightly"))]
impl<T> IsntNever for T {}

/// The user facing part of the Component traits.
///
/// This requires `ComponentBase` to be implemented, which can be done via the `#[derive(Component)]` macro.
/// ***You need both `#[derive(Component)]` and `impl Component for ...` to fully implement this
/// trait***
///
/// # Example
/// ```rust
/// #[derive(Component)]
/// struct HelloWorld;
///
/// impl Component for HelloWorld {
///     fn render() -> impl Element<Self::Data> {
///         e::h1().text("Hello World")
///     }
/// }
/// ```
///
/// See the [Reactivity](TODO) chapther in the book for information about using state in a
/// component
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a component.",
    label = "Expected Component",
    note = "`#[derive(Component)]` does not implement `Component`"
)]
pub trait Component: ComponentBase {
    /// Messages this component can emit.
    ///
    /// Use `NoMessages` if you do not need to emit any messages.
    #[cfg(feature = "nightly")]
    type EmitMessage = NoMessages;
    /// Messages this component can emit.
    ///
    /// Use `NoMessages` if you do not need to emit any messages.
    #[cfg(not(feature = "nightly"))]
    type EmitMessage;

    /// Return the root element of the component.
    ///
    /// You **can not** dirrectly reference state in this function, and should use narrowly scoped
    /// closures in the element tree instead.
    ///
    /// ```rust
    /// fn render() -> impl Element<Self::Data> {
    ///     e::h1().text(|ctx: &S<Self>| *ctx.welcome_message)
    /// }
    /// ```
    ///
    /// See the [Reactivity](TODO) chapther in the book for more info
    fn render() -> impl Element<Self>;

    /// Called when the component is mounted.
    /// Can be used to setup Effects or start async tasks.
    fn on_mount(_ctx: &mut S<Self>) {}
}

/// Wrapper around a component to let it be used as a subcomponet, `.child(C(MyComponent))`
///
/// This exsists because of type system limitations.
pub struct C<I>(pub I);

impl<I, P> Element<P> for C<I>
where
    I: Component,
    P: Component,
{
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<P>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let data = self.0.into_state();
        let element = I::render();

        let mut borrow_data = data.borrow_mut();
        I::on_mount(&mut borrow_data);

        let mut hooks = Vec::new();

        let mut state = RenderingState {
            keep_alive: render_state.keep_alive,
            hooks: &mut hooks,
            parent_dep: HookKey::default(),
        };

        let node = element.render(&mut borrow_data, &mut state);
        drop(borrow_data);
        render_state.keep_alive.push(Box::new(data));
        node
    }
}

/// The result of rendering a component
///
/// This should be kept in memory for as long as the component is in the dom.
#[must_use = "Dropping this before the component is unmounted will cause panics"]
#[expect(
    dead_code,
    reason = "This is used to keep the component alive and we do not need to use it"
)]
pub struct RenderResult<C: Component> {
    /// The component data
    data: Rc<RefCell<State<C>>>,
    /// The various things that need to be kept alive
    keep_alive: Vec<Box<dyn SmallAny>>,
}

/// Mount the specified component at natrixses default location.
/// This is what should be used when building with the natrix cli.
///
/// If the `panic_hook` feature is enabled, this will set the panic hook as well.
///
/// **WARNING:** This method implicitly leaks the memory of the root component
pub fn mount<C: Component>(component: C) {
    #[cfg(feature = "panic_hook")]
    crate::panics::set_panic_hook();

    mount_at(component, natrix_shared::MOUNT_POINT);
}

/// Mounts the component at the target id
/// Replacing the element with the component
///
/// **WARNING:** This method implicitly leaks the memory of the root component
///
/// # Panics
/// If target mount point is not found.
pub fn mount_at<C: Component>(component: C, target_id: &'static str) {
    let result = render_component(component, target_id);

    std::mem::forget(result);
}

/// Mounts the component at the target id
/// Replacing the element with the component
///
/// # Panics
/// If target mount point is not found.
#[expect(
    clippy::expect_used,
    reason = "This is the entry point of the framework, and it fails fast."
)]
pub fn render_component<C: Component>(component: C, target_id: &str) -> RenderResult<C> {
    let data = component.into_state();
    let element = C::render();

    let mut borrow_data = data.borrow_mut();
    C::on_mount(&mut borrow_data);

    let mut keep_alive = Vec::new();
    let mut hooks = Vec::new();

    let mut state = RenderingState {
        keep_alive: &mut keep_alive,
        hooks: &mut hooks,
        parent_dep: HookKey::default(),
    };
    let node = element.render(&mut borrow_data, &mut state);

    let document = get_document();
    let target = document
        .get_element_by_id(target_id)
        .expect("Failed to get mount point");
    target
        .replace_with_with_node_1(&node)
        .expect("Failed to replace mount point");

    drop(borrow_data);

    RenderResult { data, keep_alive }
}

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

    fn render() -> impl Element<Self> {
        crate::element::Comment
    }
}

/// Allows you to mark a node as non-reactive.
/// This should mainly be used when creating components that are generic over a type it wants to
/// render
///
/// As the naive solution:
/// ```rust
/// impl<T: Element<Self>> Component for MyStruct<T> {
///     // ...
/// }
/// ```
/// Causes a recursion loop in the trait analyzer, as it has to prove `Self: Component` to satisify
/// `Element<Self>`, which again tries to satifiy `Element<Self>`.
///
/// The solution is to use `Element<()>` and `NonReactive`
///
/// ```rust
/// #[derive(Component)]
/// struct MyStruct<T>(T);
///
/// impl<T: Element<()> + Copy> Component for MyStruct<T> {
///     // ...
///     fn render() -> impl Element<Self> {
///         e::div().child(|ctx: R<Self>| NonReactive(*ctx.0))
///     }
/// }
/// ```
pub struct NonReactive<E>(pub E);

impl<E: Element<()>, C: Component> Element<C> for NonReactive<E> {
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let state = State::new(());
        let mut state = state.borrow_mut();
        self.0.render(&mut state, render_state)
    }
}

impl<A: ToAttribute<()>, C: Component> ToAttribute<C> for NonReactive<A> {
    fn apply_attribute(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::Element,
        _ctx: &mut State<C>,
        rendering_state: &mut RenderingState,
    ) {
        let state = State::new(());
        let mut state = state.borrow_mut();
        Box::new(self.0).apply_attribute(name, node, &mut state, rendering_state);
    }
}
