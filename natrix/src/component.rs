//! Component traits

use std::cell::RefCell;
use std::rc::Rc;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::element::Element;
use crate::get_document;
use crate::html_elements::ToAttribute;
use crate::signal::{RenderingState, SignalMethods};
use crate::state::{ComponentData, E, HookKey, State};
use crate::utils::{SmallAny, debug_expect};

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

    /// Message that can be received by this component.
    ///
    /// Use `NoMessages` if you do not need to receive any messages.
    #[cfg(feature = "nightly")]
    type ReceiveMessage = NoMessages;
    /// Message that can be received by this component.
    ///     
    /// Use `NoMessages` if you do not need to receive any messages.
    #[cfg(not(feature = "nightly"))]
    type ReceiveMessage;

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
    /// See the [Reactivity](TODO) chapther in the book for more info
    fn render() -> impl Element<Self>;

    /// Called when the component is mounted.
    /// Can be used to setup Effects or start async tasks.
    fn on_mount(_ctx: E<Self>) {}

    /// Handle a incoming message
    /// Default implementation does nothing
    #[expect(
        unused_variables,
        reason = "We want the auto-completion for this method to be connvenient"
    )]
    fn handle_message(ctx: E<Self>, msg: Self::ReceiveMessage) {
        // This doesnt have anything to do with panic hooks
        // but `panic_hook` does pull in `web_sys::console`
        // And it feels very silly to add a cargo feature for
        // "warn_on_handle_message_not_implemented"
        // (I suppose we could also just always pull in `web_sys::console`)
        //
        // Also since the default (should) be `NoMessages` (which is `!`) this will only ever actually be called
        // If the user has a `ReceiveMessage` type that is not `NoMessages`.
        #[cfg(feature = "panic_hook")]
        web_sys::console::warn_1(
            &format!(
                "Component {} received message, but does not implement a handler",
                std::any::type_name::<Self>(),
            )
            .into(),
        );
    }
}

/// Type of the emitting message handler
type MessageHandler<P, M> = Box<dyn Fn(E<P>, M)>;

/// Trait for maybe getting a message handler
trait MaybeHandler<C: Component, M> {
    /// Get the message handler
    fn get(self) -> Option<MessageHandler<C, M>>;
}

impl<C: Component, M> MaybeHandler<C, M> for () {
    fn get(self) -> Option<MessageHandler<C, M>> {
        None
    }
}
impl<C: Component, M> MaybeHandler<C, M> for MessageHandler<C, M> {
    fn get(self) -> Option<MessageHandler<C, M>> {
        Some(self)
    }
}

/// Trait for maybe getting a message receiver
trait MaybeRecv<M> {
    /// Get the message handler
    fn get(self) -> Option<UnboundedReceiver<M>>;
}
impl<M> MaybeRecv<M> for () {
    fn get(self) -> Option<UnboundedReceiver<M>> {
        None
    }
}
impl<M> MaybeRecv<M> for UnboundedReceiver<M> {
    fn get(self) -> Option<UnboundedReceiver<M>> {
        Some(self)
    }
}

/// Wrapper around a component to let it be used as a subcomponet, `.child(C::new(MyComponent))`
///
/// This exists because of type system limitations.
#[must_use = "This is useless if not mounted"]
pub struct C<I: Component, Im, Ir> {
    /// The component data
    data: I,
    /// Message handler
    message_handler: Im,
    /// The receiver for messages
    receiver: Ir,
}

impl<I: Component> C<I, (), ()> {
    /// Create a new sub component wrapper
    pub fn new(data: I) -> Self {
        C {
            data,
            message_handler: (),
            receiver: (),
        }
    }
}
impl<I: Component, Ir> C<I, (), Ir> {
    /// Handle messages from the component
    pub fn on<P: Component>(
        self,
        handler: impl Fn(E<P>, I::EmitMessage) + 'static,
    ) -> C<I, MessageHandler<P, I::EmitMessage>, Ir> {
        C {
            data: self.data,
            message_handler: Box::new(handler),
            receiver: self.receiver,
        }
    }
}

/// Allows sending messages to the component
#[derive(Clone)]
#[must_use]
pub struct Sender<M>(UnboundedSender<M>);

impl<M> Sender<M> {
    /// Send a message to the component
    pub fn send(&self, msg: M) {
        debug_expect!(
            self.0.unbounded_send(msg),
            "Failed to send message to component"
        );
    }
}

impl<I: Component, Im> C<I, Im, ()> {
    /// Get a sender to allow sending messages to the component
    pub fn sender(
        self,
    ) -> (
        C<I, Im, UnboundedReceiver<I::ReceiveMessage>>,
        Sender<I::ReceiveMessage>,
    ) {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let comp = C {
            data: self.data,
            message_handler: self.message_handler,
            receiver: rx,
        };

        (comp, Sender(tx))
    }
}

impl<I, P, H, R> Element<P> for C<I, H, R>
where
    I: Component,
    P: Component,
    H: MaybeHandler<P, I::EmitMessage> + 'static,
    R: MaybeRecv<I::ReceiveMessage> + 'static,
{
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<P>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let data = self.data.into_state();
        let element = I::render();

        let mut borrow_data = data.borrow_mut();
        if let Some(handler) = self.message_handler.get() {
            let (tx, rx) = futures::channel::mpsc::unbounded();
            borrow_data.register_parent(tx);

            ctx.spawn_listening_task(handler, rx);
        }
        if let Some(receiver) = self.receiver.get() {
            borrow_data.spawn_recivier_task(receiver);
        }
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
/// # Panics
/// If the mount point is not found, which should never happen if using `natrix build`
#[expect(
    clippy::expect_used,
    reason = "This will never happen if `natrix build` is used, and also happens early in the app lifecycle"
)]
pub fn mount<C: Component>(component: C) {
    #[cfg(feature = "panic_hook")]
    crate::panics::set_panic_hook();

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
        .ok_or("Failed to get mount point")?;
    target
        .replace_with_with_node_1(&node)
        .map_err(|_| "Failed to replace mount point")?;

    drop(borrow_data);

    Ok(RenderResult { data, keep_alive })
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
    type ReceiveMessage = NoMessages;

    fn render() -> impl Element<Self> {
        crate::element::Comment
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
/// # use natrix::component::NonReactive;
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
