//! Component traits

use std::cell::RefCell;
use std::rc::Rc;

use crate::element::Element;
use crate::get_document;
use crate::signal::RenderingState;
use crate::state::{ComponentData, HookKey, State};

/// The base component, this is implemented by the `#[derive(Component)]` macro and handles
/// associating a component with its reactive state as well as converting to a struct to its
/// reactive counter part
#[diagnostic::on_unimplemented(
    message = "`{Self}` Missing `#[derive(Component)]`.",
    note = "`#[derive(Component)]` Required for implementing `Component`"
)]
pub trait ComponentBase: Sized {
    /// The reactive version of this struct.
    /// Should be used for most locations where a "Component" is expected.
    type Data: ComponentData;

    /// Convert this struct into its reactive variant.
    fn into_data(self) -> Self::Data;

    /// Convert this to its reactive variant and wrap it in the component state struct.
    fn into_state(self) -> Rc<RefCell<State<Self::Data>>> {
        State::new(self.into_data())
    }
}

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
    fn render() -> impl Element<Self::Data>;
}

/// Wrapper around a component to let it be used as a subcomponet, `.child(C(MyComponent))`
///
/// This exsists because of type system limitations.
pub struct C<I>(pub I);

impl<I, P> Element<P> for C<I>
where
    I: Component + 'static,
{
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<P>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let data = self.0.into_state();
        let element = I::render();

        let mut borrow_data = data.borrow_mut();

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

/// Mounts the component at the target id
/// Replacing the element with the component
/// This should be the entry point to your application
///
/// **WARNING:** This method implicitly leaks the memory of the root component
///
/// # Panics
/// If target mount point is not found.
#[inline]
#[expect(
    clippy::expect_used,
    reason = "This is the entry point of the framework, and it fails fast."
)]
pub fn mount_component<C: Component>(component: C, target_id: &'static str) {
    let data = component.into_state();
    let element = C::render();

    let mut borrow_data = data.borrow_mut();

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

    // This is the entry point, this component should be alive FOREVER
    std::mem::forget(data);
    std::mem::forget(keep_alive);
}
