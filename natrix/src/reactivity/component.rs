//! Component traits

use std::cell::RefCell;
use std::rc::Rc;

use crate::dom::element::{
    DynElement,
    Element,
    ElementRenderResult,
    MaybeStaticElement,
    generate_fallback_node,
};
use crate::error_handling::debug_panic;
use crate::get_document;
use crate::reactivity::signal::RenderingState;
use crate::reactivity::state::{
    ComponentData,
    E,
    EagerMessageSender,
    EventToken,
    HookKey,
    KeepAlive,
    State,
};

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
pub enum NoMessages {}

/// The user facing part of the Component traits.
///
/// This requires `ComponentBase` to be implemented, which can be done via the `#[derive(Component)]` macro.
/// ***You need both `#[derive(Component)]` and `impl Component for ...` to fully implement this
/// trait***
///
/// # Example
/// ```rust
/// # use natrix::prelude::*;
/// #[derive(Component)]
/// struct HelloWorld;
///
/// impl Component for HelloWorld {
///     type EmitMessage = NoMessages;
///     type ReceiveMessage = NoMessages;
///     fn render() -> impl Element<Self> {
///         e::h1().text("Hello World")
///     }
/// }
/// ```
///
/// See the [Reactivity](https://vivax3794.github.io/natrix/reactivity.html) chapther in the book for information about using state in a
/// component
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a component.",
    label = "Expected Component",
    note = "`#[derive(Component)]` does not implement `Component`"
)]
pub trait Component: ComponentBase {
    cfg_if::cfg_if! {
        if #[cfg(feature = "nightly")] {
            /// Messages this component can emit.
            ///
            /// Use `NoMessages` if you do not need to emit any messages.
            /// Defaults to `NoMessages` on nightly.
            type EmitMessage = NoMessages;
        } else {
            /// Messages this component can emit.
            ///
            /// Use `NoMessages` if you do not need to emit any messages.
            type EmitMessage;
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "nightly")] {
            /// Message that can be received by this component.
            ///
            /// Use `NoMessages` if you do not need to receive any messages.
            /// Defaults to `NoMessages` on nightly.
            type ReceiveMessage = NoMessages;
        } else {
            /// Message that can be received by this component.
            ///
            /// Use `NoMessages` if you do not need to receive any messages.
            type ReceiveMessage;
        }
    }

    /// Return the root element of the component.
    ///
    /// You **can not** directly reference state in this function, and should use narrowly scoped
    /// closures in the element tree instead.
    ///
    /// ```rust
    /// # use natrix::prelude::*;
    /// # #[derive(Component)]
    /// # struct HelloWorld {welcome_message: &'static str};
    /// # impl Component for HelloWorld {
    /// #     type EmitMessage = NoMessages;
    /// #     type ReceiveMessage = NoMessages;
    /// fn render() -> impl Element<Self> {
    ///     e::h1().text(|ctx: R<Self>| *ctx.welcome_message)
    /// }
    /// # }
    /// ```
    ///
    /// See the [Reactivity](https://vivax3794.github.io/natrix/reactivity.html) chapther in the book for more info
    fn render() -> impl Element<Self>;

    /// Called when the component is mounted.
    /// Can be used to setup Effects or start async tasks.
    #[expect(
        unused_variables,
        reason = "We want the auto-completion for this method to be connvenient"
    )]
    fn on_mount(ctx: E<Self>, token: EventToken) {}

    /// Handle a incoming message
    /// Default implementation does nothing
    #[expect(
        unused_variables,
        reason = "We want the auto-completion for this method to be connvenient"
    )]
    fn handle_message(ctx: E<Self>, msg: Self::ReceiveMessage, token: EventToken) {
        log::warn!(
            "Component {} received message, but does not implement a handler",
            std::any::type_name::<Self>()
        );
    }
}

/// Maybe a handler
trait MaybeHandler<P: Component, C: Component> {
    /// Setup the handler if it is present
    fn apply(self, ctx: &mut State<P>, this: &mut State<C>);
}

impl<P: Component, C: Component> MaybeHandler<P, C> for () {
    fn apply(self, _ctx: &mut State<P>, _this: &mut State<C>) {}
}

impl<P: Component, C: Component, F> MaybeHandler<P, C> for F
where
    F: Fn(E<P>, C::EmitMessage, EventToken),
    F: Clone + 'static,
{
    fn apply(self, ctx: &mut State<P>, this: &mut State<C>) {
        let emit = ctx.emit_sender(self);
        this.register_parent(emit);
    }
}

/// Wrapper around a component to let it be used as a subcomponet, `.child(C::new(MyComponent))`
///
/// This exists because of type system limitations.
#[must_use = "This is useless if not mounted"]
pub struct SubComponent<I: Component, Handler> {
    /// The component data
    data: Rc<RefCell<State<I>>>,
    /// Handler for out emitted messages
    handler: Handler,
}

impl<I: Component> SubComponent<I, ()> {
    /// Create a new sub component wrapper
    #[inline]
    pub fn new(data: I) -> Self {
        SubComponent {
            data: State::new(data.into_data()),
            handler: (),
        }
    }
}
impl<I: Component> SubComponent<I, ()> {
    /// Handle messages from the component
    #[inline]
    pub fn on<P, F>(self, handler: F) -> SubComponent<I, F>
    where
        P: Component,
        F: Fn(E<P>, I::EmitMessage, EventToken) + 'static + Clone,
    {
        SubComponent {
            data: self.data,
            handler,
        }
    }
}

/// Allows sending messages to the component
#[must_use]
pub struct Sender<C: Component>(EagerMessageSender<C>);

impl<C: Component> Clone for Sender<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C: Component> Sender<C> {
    /// Send a message to the component
    #[inline]
    pub fn send(&self, msg: C::ReceiveMessage, _token: EventToken) {
        let result = self.0.send(super::state::InternalMessage::FromParent(msg));
        if result.is_none() {
            log::warn!("Sending message to unmounted componen");
        }
    }
}

impl<I: Component, Handler> SubComponent<I, Handler> {
    /// Get a sender to allow sending messages to the component
    #[inline]
    pub fn sender(&self) -> Sender<I> {
        if let Ok(data) = self.data.try_borrow() {
            let eager = data.eager_sender();
            Sender(eager)
        } else {
            debug_panic!("State already borrowed during construction");
            Sender(EagerMessageSender::create_closed_fallback())
        }
    }
}

impl<I, P, Handler> DynElement<P> for SubComponent<I, Handler>
where
    I: Component,
    P: Component,
    Handler: MaybeHandler<P, I>,
{
    fn render(
        self: Box<Self>,
        ctx: &mut State<P>,
        render_state: &mut RenderingState,
    ) -> ElementRenderResult {
        let data = self.data;
        let element = I::render();

        let Ok(mut borrow_data) = data.try_borrow_mut() else {
            debug_panic!("State already borrowed during construction");
            return ElementRenderResult::Node(generate_fallback_node());
        };

        self.handler.apply(ctx, &mut borrow_data);

        I::on_mount(&mut borrow_data, EventToken::new());

        let mut hooks = Vec::new();

        let mut state = RenderingState {
            keep_alive: render_state.keep_alive,
            hooks: &mut hooks,
            parent_dep: HookKey::default(),
        };

        let node = element.into_generic().render(&mut borrow_data, &mut state);
        drop(borrow_data);
        render_state.keep_alive.push(Box::new(data));
        node
    }
}

impl<I, P, Handler> Element<P> for SubComponent<I, Handler>
where
    I: Component,
    P: Component,
    Handler: 'static,
    Self: DynElement<P>,
{
    fn into_generic(self) -> MaybeStaticElement<P> {
        MaybeStaticElement::Dynamic(Box::new(self))
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
    keep_alive: Vec<KeepAlive>,
}

/// Mount the specified component at natrixses default location.
/// This is what should be used when building with the natrix cli.
///
/// IMPORTANT: This is the intended entry point for `natrix-cli` build applications, and the natrix
/// cli build system expects this to be called.
///
/// **WARNING:** This method implicitly leaks the memory of the root component
/// # Panics
/// If the mount point is not found, which should never happen if using `natrix build`
#[expect(
    clippy::expect_used,
    reason = "This will never happen if `natrix build` is used, and also happens early in the app lifecycle"
)]
pub fn mount<C: Component>(component: C) {
    crate::panics::set_panic_hook();

    #[cfg(feature = "console_log")]
    if cfg!(target_arch = "wasm32") {
        if let Err(err) = console_log::init_with_level(log::Level::Trace) {
            crate::error_handling::debug_panic!("Failed to create logger: {err}");
        }
    }
    #[cfg(feature = "_internal_extract_css")]
    if let Err(err) = simple_logger::init_with_level(log::Level::Trace) {
        eprintln!("Failed to setup logger {err}");
    }
    log::info!("Logging initialized");

    #[cfg(feature = "_internal_collect_css")]
    crate::css::css_collect();

    if cfg!(feature = "_internal_extract_css") {
        log::info!("Css extract mode, aboring mount.");
        return;
    }

    mount_at(component, natrix_shared::MOUNT_POINT).expect("Failed to mount");
}

/// Mounts the component at the target id
/// Replacing the element with the component
///
/// **WARNING:** This method implicitly leaks the memory of the root component
///
/// # Errors
/// If target mount point is not found.
pub fn mount_at<C: Component>(component: C, target_id: &'static str) -> Result<(), &'static str> {
    let result = render_component(component, target_id)?;

    std::mem::forget(result);
    Ok(())
}

/// Mounts the component at the target id
/// Replacing the element with the component
/// # Errors
/// If target mount point is not found.
pub fn render_component<C: Component>(
    component: C,
    target_id: &str,
) -> Result<RenderResult<C>, &'static str> {
    log::info!(
        "Mounting root component {} at #{target_id}",
        std::any::type_name::<C>()
    );
    let data = component.into_state();
    let element = C::render();

    let mut borrow_data = data.borrow_mut();
    C::on_mount(&mut borrow_data, EventToken::new());

    let mut keep_alive = Vec::new();
    let mut hooks = Vec::new();

    let mut state = RenderingState {
        keep_alive: &mut keep_alive,
        hooks: &mut hooks,
        parent_dep: HookKey::default(),
    };
    let node = element
        .into_generic()
        .render(&mut borrow_data, &mut state)
        .into_node();

    let document = get_document();
    let target = document
        .get_element_by_id(target_id)
        .ok_or("Failed to get mount point")?;
    target
        .replace_with_with_node_1(&node)
        .map_err(|_| "Failed to replace mount point")?;

    drop(borrow_data);

    Ok(RenderResult { data, keep_alive })
}
