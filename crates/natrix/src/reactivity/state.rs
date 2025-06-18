//! Types for handling the component state

use std::any::Any;
use std::cell::RefCell;
use std::collections::BinaryHeap;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use slotmap::{SlotMap, new_key_type};
use smallvec::SmallVec;

use crate::error_handling::log_or_panic;
use crate::reactivity::component::Component;
use crate::reactivity::render_callbacks::{DummyHook, ReactiveHook, RenderingState, UpdateResult};
use crate::reactivity::signal::SignalMethods;

/// Trait automatically implemented on reactive structs by the derive macro.
///
/// This trait provides the internal interface between component data structures
/// and the reactive system. It allows the framework to:
/// - Access all reactive signals within a component
/// - Capture the current state of all signals
/// - Restore signals to a previously captured state
///
/// This is an internal trait not meant for direct implementation by users.
#[doc(hidden)]
pub trait ComponentData: Sized + 'static {
    /// References to all reactive signals in this component.
    ///
    /// This is typically implemented as an array of mutable references to the component's
    /// signal fields, allowing the reactive system to track and update them.
    type FieldRef<'a>: IntoIterator<Item = &'a mut dyn SignalMethods>;

    /// A complete snapshot of all signal values in this component.
    ///
    /// This type captures the entire signal state for later restoration,
    /// typically used for nested reactive contexts such as `.watch`.
    type SignalState;

    /// Returns mutable references to all signals in this component.
    ///
    /// This allows the reactive system to track modifications and trigger
    /// updates when signal values change.
    fn signals_mut(&mut self) -> Self::FieldRef<'_>;

    /// Extracts the current signal state and resets signals to their default state.
    fn pop_signals(&mut self) -> Self::SignalState;

    /// Restores all signals to a previously captured state.
    fn set_signals(&mut self, state: Self::SignalState);
}

/// for keeping specific objects alive in memory such as `Closure` and `Rc`
pub(crate) type KeepAlive = Box<dyn Any>;

new_key_type! { pub(crate) struct HookKey; }

/// A token only accessible in events.
/// This is used to guard certain apis that should only be used in events.
#[derive(Clone, Copy)]
pub struct EventToken {
    /// A private field to prevent this from being constructed outside of the framework
    _private: (),
}

impl EventToken {
    /// Create a new token
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

/// A message on the internal messing system
pub(crate) enum InternalMessage<C: Component> {
    /// Message from parent
    FromParent(C::ReceiveMessage),
    /// Message from a child
    FromChild(Box<dyn FnOnce(&mut State<C>)>),
}

impl<C: Component> std::fmt::Debug for InternalMessage<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FromParent(_msg) => f
                .debug_tuple("InternalMessage::FromParent")
                .finish_non_exhaustive(),
            Self::FromChild(_msg) => f
                .debug_tuple("InternalMessage::FromChild")
                .finish_non_exhaustive(),
        }
    }
}

/// The queue of messages sent to a component while it was borrowed
type DeferredMessageQueue<C> = RefCell<SmallVec<[InternalMessage<C>; 1]>>;

/// Send messages to a component which are executed right away if component is not borrowed.
/// If the component is borrowed its assumed we are in a recursive event context and the messages
/// are appended to a queue.
/// The component should check this queue on its next update call.
pub(crate) struct EagerMessageSender<C: Component> {
    /// A direct reference to the state
    direct: Weak<RefCell<State<C>>>,
    /// A reference to the message queue
    deferred: Weak<DeferredMessageQueue<C>>,
}

impl<C: Component> Clone for EagerMessageSender<C> {
    fn clone(&self) -> Self {
        Self {
            direct: Weak::clone(&self.direct),
            deferred: Weak::clone(&self.deferred),
        }
    }
}

/// A function that can be used to emit a message of the given type to the parent.
type EmitMessageSender<Msg> = Box<dyn Fn(Vec<Msg>)>;

impl<C: Component> EagerMessageSender<C> {
    /// Create a closed channel, used as a fallback when hitting error during construction
    /// (in order to satisfy return types in release mode)
    pub(crate) fn create_closed_fallback() -> Self {
        Self {
            direct: Weak::new(),
            deferred: Weak::new(),
        }
    }

    /// Send a message to the channel.
    /// return `None` if channel closed.
    pub(crate) fn send(&self, message: InternalMessage<C>) -> Option<()> {
        self.send_batched(std::iter::once(message))
    }

    /// Send multiple messages at once.
    /// This method avoids the overhead of multiple `RefCell` checks and reactive updates.
    pub(crate) fn send_batched(
        &self,
        messages: impl IntoIterator<Item = InternalMessage<C>>,
    ) -> Option<()> {
        let messages = messages.into_iter();
        let direct = self.direct.upgrade()?;

        if let Ok(mut direct) = direct.try_borrow_mut() {
            log::trace!("Handling message immediately");

            direct.clear();
            for message in messages {
                direct.handle_message(message);
            }
            direct.update();
        } else {
            log::debug!("Recursive event handling detected, deferring handling of message");

            let deferred = self.deferred.upgrade()?;
            let Ok(mut deferred) = deferred.try_borrow_mut() else {
                log_or_panic!("Failed to borrow deferred message queue");
                return None;
            };

            deferred.extend(messages);
        }

        Some(())
    }
}

/// Store some data but use `O` for its `Ord` implementation
struct OrderAssociatedData<T, O> {
    /// The data in question
    data: T,
    /// The value to order based on
    ordering: O,
}

impl<T, O: PartialEq> PartialEq for OrderAssociatedData<T, O> {
    fn eq(&self, other: &Self) -> bool {
        self.ordering == other.ordering
    }
}
impl<T, O: Eq> Eq for OrderAssociatedData<T, O> {}

impl<T, O: PartialOrd> PartialOrd for OrderAssociatedData<T, O> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ordering.partial_cmp(&other.ordering)
    }
}
impl<T, O: Ord> Ord for OrderAssociatedData<T, O> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordering.cmp(&other.ordering)
    }
}

/// The core component state, stores all framework data
pub struct State<T: Component> {
    /// The user (macro) defined reactive struct
    pub(crate) data: T::Data,
    /// A weak reference to ourself, so that event handlers can easially get a weak reference
    /// without having to pass it around in every api
    pub(crate) this: Weak<RefCell<Self>>,
    /// Reactive hooks
    hooks: SlotMap<HookKey, (Box<dyn ReactiveHook<T>>, u64)>,
    /// The next value to use in the insertion order map
    next_insertion_order_value: u64,
    /// Messages gotten while we were borrowed
    deferred_messages: Rc<DeferredMessageQueue<T>>,
    /// Emitting to the parent
    to_parent_emit: Option<EmitMessageSender<T::EmitMessage>>,
}

impl<T: Component> Deref for State<T> {
    type Target = T::Data;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Component> DerefMut for State<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// A type alias for `&mut State<C>`, should be preferred in closure argument hints.
/// such as `|ctx: E<Self>| ...`
pub type E<'c, C> = &'c mut State<C>;

/// A type alias for `&mut RenderCtx<C>`, should be preferred in closure argument hints.
/// such as `|ctx: R<Self>| ...`
pub type R<'a, 'c, C> = &'a mut RenderCtx<'c, C>;

impl<T: Component> State<T> {
    /// Create a minimal instance of this without wrapping in Self
    ///
    /// Warning the `Weak` reference is not set up yet
    pub(crate) fn create_base(data: T::Data) -> Self {
        Self {
            data,
            this: Weak::new(),
            hooks: SlotMap::default(),
            next_insertion_order_value: 0,
            deferred_messages: Rc::default(),
            to_parent_emit: None,
        }
    }

    /// Convert this into a finlized state by populating `Weak` and returning a Rc
    pub(crate) fn finalize(self) -> Rc<RefCell<Self>> {
        let this = Rc::new(RefCell::new(self));

        if let Ok(mut borrow) = this.try_borrow_mut() {
            borrow.this = Rc::downgrade(&this);
        } else {
            log_or_panic!("State (somehow) already borrowed in `finalize");
        }

        this
    }

    /// Create a new instance of the state, returning a `Rc` to it
    pub(crate) fn new(data: T::Data) -> Rc<RefCell<Self>> {
        Self::create_base(data).finalize()
    }

    /// Handle a internal message
    ///
    /// This does not trigger clean reactive tracking or updates of the action
    /// This is to allow batching messages handling.
    /// Calling `.clear` and `.update` is meant for the caller
    fn handle_message(&mut self, message: InternalMessage<T>) {
        log::debug!("Handling message {message:?}");
        match message {
            InternalMessage::FromParent(message) => {
                T::handle_message(self, message, EventToken::new());
            }
            InternalMessage::FromChild(handler) => handler(self),
        }
    }

    /// Clear out the deferred message queue
    ///
    /// This does not call `.clear` or `.update`,
    /// As this is meant to be used at the start of `.update` itself.
    fn drain_message_queue(&mut self) {
        let queue = if let Ok(mut queue) = self.deferred_messages.try_borrow_mut() {
            if queue.is_empty() {
                log::trace!("No messages to process");
                return;
            }

            // We create a new vec with the same size because thats likely the capacity that will
            // be needed in the future as well.
            let mut new_vec = SmallVec::with_capacity(queue.len());
            // We do this instead of a drain because handling a message can lead to us receiving
            // more deferred messages.
            std::mem::swap(&mut new_vec, &mut *queue);
            new_vec
        } else {
            log_or_panic!("Message queue already borrowed while in drain_message_queue");
            return;
        };

        log::debug!("Processing {} deferred messages", queue.len());
        for message in queue {
            self.handle_message(message);
        }

        // Ensure any messages queued because of the above handling are handled as well
        self.drain_message_queue();
    }

    /// Get a `EagerMessageSender` to this component
    pub(crate) fn eager_sender(&self) -> EagerMessageSender<T> {
        EagerMessageSender {
            direct: self.this.clone(),
            deferred: Rc::downgrade(&self.deferred_messages),
        }
    }

    /// Get a `EmitMessageSender` for this component with the given message type
    pub(crate) fn emit_sender<M, F>(&self, handler: F) -> EmitMessageSender<M>
    where
        F: Fn(&mut Self, M, EventToken) + 'static + Clone,
        M: 'static,
    {
        let eager = self.eager_sender();
        Box::new(move |messages: Vec<M>| {
            let handle_clone = handler.clone();
            let command = move |this: &mut Self| {
                for message in messages {
                    handle_clone(this, message, EventToken::new());
                }
            };
            eager.send(InternalMessage::FromChild(Box::new(command)));
        })
    }

    /// Clear all signals
    pub(crate) fn clear(&mut self) {
        for signal in self.data.signals_mut() {
            signal.clear();
        }
    }

    /// Insert a hook
    pub(crate) fn insert_hook(&mut self, hook: Box<dyn ReactiveHook<T>>) -> HookKey {
        let key = self.hooks.insert((hook, self.next_insertion_order_value));
        self.next_insertion_order_value =
            if let Some(value) = self.next_insertion_order_value.checked_add(1) {
                value
            } else {
                log_or_panic!("Insertion order overflow");
                0
            };
        key
    }

    /// Update the value for a hook
    pub(crate) fn set_hook(&mut self, key: HookKey, hook: Box<dyn ReactiveHook<T>>) {
        if let Some(slot) = self.hooks.get_mut(key) {
            slot.0 = hook;
        }
    }

    /// Register a dependency for all read signals
    pub(crate) fn reg_dep(&mut self, dep: HookKey) {
        for signal in self.data.signals_mut() {
            signal.register_dep(dep);
        }
    }

    /// Remove the hook from the slotmap, runs the function on it, then puts it back.
    ///
    /// This is to allow mut access to both the hook and self, which is required by most hooks.
    /// (and yes hooks also mutable access the slotmap while running)
    fn run_with_hook_and_self<F, R>(&mut self, hook: HookKey, func: F) -> Option<R>
    where
        F: FnOnce(&mut Self, &mut Box<dyn ReactiveHook<T>>) -> R,
    {
        let slot_ref = self.hooks.get_mut(hook)?;
        let mut temp_hook: Box<dyn ReactiveHook<T>> = Box::new(DummyHook);
        std::mem::swap(&mut slot_ref.0, &mut temp_hook);

        let res = func(self, &mut temp_hook);

        let slot_ref = self.hooks.get_mut(hook)?;
        slot_ref.0 = temp_hook;

        Some(res)
    }

    /// Loop over signals and update any depdant hooks for changed signals
    /// This also drains the deferred message queue
    pub(crate) fn update(&mut self) {
        log::debug!("Performing update cycle for {}", std::any::type_name::<T>());
        self.drain_message_queue();

        let mut hooks = BinaryHeap::new();
        for signal in self.data.signals_mut() {
            if signal.changed() {
                for dep in signal.drain_dependencies() {
                    let dep_insertion_order = self.hooks.get(dep).map(|x| x.1).unwrap_or_default();
                    hooks.push(OrderAssociatedData {
                        data: dep,
                        ordering: std::cmp::Reverse(dep_insertion_order),
                    });
                }
            }
        }

        log::trace!("{} hooks updating", hooks.len());
        while let Some(OrderAssociatedData { data: hook_key, .. }) = hooks.pop() {
            self.run_with_hook_and_self(hook_key, |ctx, hook| match hook.update(ctx, hook_key) {
                UpdateResult::Nothing => {}
                UpdateResult::RunHook(dep) => {
                    hooks.push(OrderAssociatedData {
                        data: dep,
                        ordering: std::cmp::Reverse(u64::MIN), // This item should be the next item
                    });
                }
                UpdateResult::DropHooks(deps) => {
                    for dep in deps {
                        drop_hook(ctx, dep);
                    }
                }
            });
        }
        log::trace!("Update cycle complete");
    }

    /// Get the unwrapped data referenced by this guard
    #[inline]
    pub fn get<'s, F, R>(&'s self, guard: &Guard<F>) -> &'s R
    where
        F: Fn(&'s Self) -> &'s R,
    {
        (guard.getter)(self)
    }

    /// Get the unwrapped data referenced by this guard, but owned
    #[inline]
    pub fn get_owned<F, R>(&self, guard: &Guard<F>) -> R
    where
        F: Fn(&Self) -> R,
    {
        (guard.getter)(self)
    }

    /// Get the unwrapped data referenced by this guard, but mut
    #[inline]
    pub fn get_mut<'s, F, R>(&'s mut self, guard: &Guard<F>) -> &'s mut R
    where
        F: Fn(&'s mut Self) -> &'s mut R,
    {
        (guard.getter)(self)
    }

    /// Emit a message to the parent component
    pub fn emit(&self, msg: T::EmitMessage, token: EventToken) {
        self.emit_batch(vec![msg], token);
    }

    /// Emit multiple messages to the parent component
    /// This is more efficient than induvidual `emit` calls.
    pub fn emit_batch(&self, msg: impl IntoIterator<Item = T::EmitMessage>, _token: EventToken) {
        if let Some(sender) = self.to_parent_emit.as_ref() {
            sender(msg.into_iter().collect());
        } else {
            log::trace!("Message emitted but no parent listener.");
        }
    }

    /// Register a new sender from the parent component
    pub(crate) fn register_parent(&mut self, sender: EmitMessageSender<T::EmitMessage>) {
        if self.to_parent_emit.is_some() {
            log_or_panic!("`to_parent_emit` set twice");
        }

        self.to_parent_emit = Some(sender);
    }

    /// Get a wrapper around `Weak<RefCell<T>>` which provides a safer api that aligns with
    /// framework assumptions.
    #[cfg(feature = "async")]
    pub fn deferred_borrow(&self, _token: EventToken) -> DeferredCtx<T> {
        DeferredCtx {
            inner: self.this.clone(),
        }
    }

    /// Spawn a async task in the local event loop, which will run on the next possible moment.
    #[cfg(feature = "async")]
    pub fn use_async<C, F>(&self, token: EventToken, func: C)
    where
        C: FnOnce(DeferredCtx<T>) -> F,
        F: Future<Output = Option<()>> + 'static,
    {
        let deferred = self.deferred_borrow(token);
        let future = func(deferred);
        let future = async {
            let _ = future.await;
        };

        let future = PanicCheckFuture { inner: future };

        wasm_bindgen_futures::spawn_local(future);
    }
}

/// Drop all children of the hook
fn drop_hook<T: Component>(ctx: &mut State<T>, hook: HookKey) {
    if let Some(hook) = ctx.hooks.remove(hook) {
        let mut hooks = hook.0.drop_us();
        for hook in hooks.drain(..) {
            drop_hook(ctx, hook);
        }
    }
}

/// Wrapper around a mutable state that only allows read-only access
///
/// This holds a mutable state to facilitate a few rendering features such as `.watch`
pub struct RenderCtx<'c, C: Component> {
    /// The inner context
    pub(crate) ctx: &'c mut State<C>,
    /// The render state for this state
    pub(crate) render_state: RenderingState<'c>,
}

impl<C: Component> Deref for RenderCtx<'_, C> {
    type Target = State<C>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<C: Component> RenderCtx<'_, C> {
    /// Calculate the value using the function and cache it using `clone`.
    /// Then whenever any signals read in the function are modified re-run the function and check
    /// if the new result is different.
    /// Only reruns the caller when the item is different.
    ///
    /// # Example
    /// ```rust
    /// # use natrix::prelude::*;
    /// # #[derive(Component)]
    /// # struct MyComponent {value: u32}
    /// #
    /// # impl Component for MyComponent {
    /// # type EmitMessage = NoMessages;
    /// # type ReceiveMessage = NoMessages;
    /// # fn render() -> impl Element<Self> {
    /// # |ctx: R<Self>| {
    /// if ctx.watch(|ctx| *ctx.value > 2) {
    ///     e::div().text(|ctx: R<Self>| *ctx.value)
    /// } else {
    ///     e::div().text("Value is too low")
    /// }
    /// # }}}
    /// ```
    #[inline]
    pub fn watch<T, F>(&mut self, func: F) -> T
    where
        F: Fn(&State<C>) -> T + 'static,
        T: PartialEq + Clone + 'static,
    {
        self.watch_mut(move |ctx| func(ctx))
    }

    /// Internal only version of mutable version of watch
    #[doc(hidden)]
    pub fn watch_mut<T, F>(&mut self, func: F) -> T
    where
        F: Fn(&mut State<C>) -> T + 'static,
        T: PartialEq + Clone + 'static,
    {
        let signal_state = self.ctx.pop_signals();

        let result = func(self.ctx);

        let hook = WatchState {
            calc_value: Box::new(func),
            last_value: result.clone(),
            dep: self.render_state.parent_dep,
        };
        let me = self.ctx.insert_hook(Box::new(hook));
        self.ctx.reg_dep(me);
        self.render_state.hooks.push(me);

        self.ctx.set_signals(signal_state);

        result
    }

    /// Get a readonly reference from a mut guard
    #[inline]
    pub fn get_downgrade<F, R>(&mut self, guard: &Guard<F>) -> &R
    where
        F: Fn(&mut State<C>) -> &mut R,
    {
        (guard.getter)(self.ctx)
    }
}

/// The wather hook / signal
struct WatchState<F, T> {
    /// Function to calculate the state
    calc_value: F,
    /// The previous cached value
    last_value: T,
    /// The dependency that owns us.
    dep: HookKey,
}

impl<C, F, T> ReactiveHook<C> for WatchState<F, T>
where
    C: Component,
    T: PartialEq,
    F: Fn(&mut State<C>) -> T,
{
    fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
        ctx.clear();
        let new_value = (self.calc_value)(ctx);
        ctx.reg_dep(you);

        if new_value == self.last_value {
            UpdateResult::Nothing
        } else {
            UpdateResult::RunHook(self.dep)
        }
    }

    fn drop_us(self: Box<Self>) -> Vec<HookKey> {
        Vec::new()
    }
}

/// Get a guard handle that can be used to retrieve the `Some` variant of a option without having to
/// use `.unwrap`.
/// Should be used to achieve find-grained reactivity (internally this uses `.watch` on `.is_some()`)
///
/// # Why?
/// The usecase can be seen by considering this logic:
/// ```rust
/// # use natrix::prelude::*;
/// # #[derive(Component)]
/// # struct MyComponent {value: Option<u32>}
/// # impl Component for MyComponent {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value) = *ctx.value {
///     e::div().text(value)
/// } else {
///     e::div().text("Is none")
/// }
/// # }}}
/// ```
/// The issue here is that the outer div (which might be a more expensive structure to create) is
/// recreated everytime `value` changes, even if it is `Some(0) -> Some(1)`
/// This is where you might reach for `ctx.watch`, and in fact that works perfectly:
/// ```rust
/// # use natrix::prelude::*;
/// # #[derive(Component)]
/// # struct MyComponent {value: Option<u32>}
/// # impl Component for MyComponent {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if ctx.watch(|ctx| ctx.value.is_some()) {
///     e::div().text(|ctx: R<Self>| ctx.value.unwrap())
/// } else {
///     e::div().text("Is none")
/// }
/// # }}}
/// ```
/// And this works, Now a change from `Some(0)` to `Some(1)` will only run the inner closure and
/// the outer div is reused. but there is one downside, we need `.unwrap` because the inner closure is
/// technically isolated, and this is ugly, and its easy to do by accident. and you might forget
/// the outer condition.
///
/// This is where guards come into play:
/// ```rust
/// # use natrix::prelude::*;
/// # use natrix::guard_option;
/// # #[derive(Component)]
/// # struct MyComponent {value: Option<u32>}
/// # impl Component for MyComponent {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value_guard) = guard_option!(|ctx| ctx.value.as_ref()) {
///     e::div().text(move |ctx: R<Self>| *ctx.get(&value_guard))
/// } else {
///     e::div().text("Is none")
/// }
/// # }}}
/// ```
/// Here `value_guard` is actually not the value at all, its a lightweight value thats can be
/// captured by child closures and basically is a way to say "I know that in this context this
/// value is `Some`"
///
/// Internally this uses `ctx.watch` and `.unwrap` (which should never fail)
///
/// ## Mutable returns
/// If you want to return a mutable reference to the value you can use the `@mut` version:
/// ```rust
/// # use natrix::prelude::*;
/// # use natrix::guard_option;
/// # #[derive(Component)]
/// # struct MyComponent {value: Option<u32>}
/// # impl Component for MyComponent {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value_guard) = guard_option!(@mut |ctx| ctx.value.as_mut()) {
///   e::button().on::<events::Click>(move |ctx: E<Self>, _, _| {
///     *ctx.get_mut(&value_guard) += 1;
///   }).generic()
/// } else {
///   e::div().text("Is none").generic()
/// }
/// # }}}
/// ```
///
/// You can also use a mutable guard in reactive closures via `get_downgrade`
/// ```rust
/// # use natrix::prelude::*;
/// # use natrix::guard_option;
/// # #[derive(Component)]
/// # struct MyComponent {value: Option<u32>}
/// # impl Component for MyComponent {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value_guard) = guard_option!(@mut |ctx| ctx.value.as_mut()) {
///   e::button()
///     .text(move |ctx: R<Self>| *ctx.get_downgrade(&value_guard))
///     .on::<events::Click>(move |ctx: E<Self>, _, _| {
///       *ctx.get_mut(&value_guard) += 1;
///     })
///     .generic()
/// } else {
///   e::div().text("Is none").generic()
/// }
/// # }}}
/// ```
///
/// **IMPORTANT**: Even tho the guard closure takes a mutable reference, you should not mutate it.
/// Instead it should be only be used to get a `&mut ...` to value you want.
///
/// ## Owned returns
/// By default this macro assumes the return value is `&T`, but if you want to return an owned
/// value you can use the `@owned` version:
/// ```rust
/// # use natrix::prelude::*;
/// # use natrix::guard_option;
/// # #[derive(Component)]
/// # struct MyComponent {value: Option<u32>}
/// # impl Component for MyComponent {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value_guard) = guard_option!(@owned |ctx| ctx.value) {
///    e::div().text(move |ctx: R<Self>| ctx.get_owned(&value_guard))
/// } else {
///    e::div().text("Is none")
/// }
/// # }}}
/// ```
#[macro_export]
macro_rules! guard_option {
    (| $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_some()) {
            Some($crate::macro_ref::Guard::new::<Self, _>(move |$ctx| {
                $expr.expect("Guard used on None value")
            }))
        } else {
            None
        }
    };
    (@mut | $ctx:ident | $expr:expr) => {
        if $ctx.watch_mut(move |$ctx| $expr.is_some()) {
            Some($crate::macro_ref::Guard::new_mut::<Self, _>(move |$ctx| {
                $expr.expect("Guard used on None value")
            }))
        } else {
            None
        }
    };
    (@owned | $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_some()) {
            Some($crate::macro_ref::Guard::new_owned::<Self, _>(
                move |$ctx| $expr.expect("Guard used on None value"),
            ))
        } else {
            None
        }
    };
}

/// Get a guard handle that can be used to retrieve the `Ok` variant of a option without having to
/// use `.unwrap`, or the `Err` variant.
#[macro_export]
macro_rules! guard_result {
    (| $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_ok()) {
            Ok($crate::macro_ref::Guard::new::<Self, _>(move |$ctx| {
                $expr.expect("Guard used on Err value")
            }))
        } else {
            Err($crate::macro_ref::Guard::new::<Self, _>(move |$ctx| {
                $expr.expect_err("Guard used on Ok value")
            }))
        }
    };
    (@mut | $ctx:ident | $expr:expr) => {
        if $ctx.watch_mut(move |$ctx| $expr.is_ok()) {
            Ok($crate::macro_ref::Guard::new_mut::<Self, _>(move |$ctx| {
                $expr.expect("Guard used on Err value")
            }))
        } else {
            Err($crate::macro_ref::Guard::new_mut::<Self, _>(move |$ctx| {
                $expr.expect_err("Guard used on Ok value")
            }))
        }
    };
    (@owned | $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_ok()) {
            Ok($crate::macro_ref::Guard::new_owned::<Self, _>(
                move |$ctx| $expr.expect("Guard used on Err value"),
            ))
        } else {
            Err($crate::macro_ref::Guard::new_owned::<Self, _>(
                move |$ctx| $expr.expect_err("Guard used on Ok value"),
            ))
        }
    };
}
/// This guard ensures that when it is in scope the data it was created for is `Some`
#[cfg_attr(feature = "nightly", must_not_suspend)]
#[derive(Clone, Copy)]
#[must_use]
pub struct Guard<F> {
    /// The closure for getting the value from a ctx
    getter: F,
}

impl<F> Guard<F> {
    #[doc(hidden)]
    #[inline]
    pub fn new<C, R>(getter: F) -> Self
    where
        F: for<'a> Fn(&'a State<C>) -> &'a R,
        C: Component,
    {
        Self { getter }
    }

    #[doc(hidden)]
    #[inline]
    pub fn new_mut<C, R>(getter: F) -> Self
    where
        F: for<'a> Fn(&'a mut State<C>) -> &'a mut R,
        C: Component,
    {
        Self { getter }
    }

    #[doc(hidden)]
    #[inline]
    pub fn new_owned<C, R>(getter: F) -> Self
    where
        F: Fn(&State<C>) -> R,
        C: Component,
    {
        Self { getter }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        use std::marker::PhantomData;
        use std::cell::RefMut;

        /// A wrapper future that checks `has_panicked` before resolving.
        ///
        /// If you are using `wasm_bindgen_futures` directly you should wrap your futures in this.
        #[pin_project::pin_project]
        pub struct PanicCheckFuture<F> {
            /// The future to run
            #[pin]
            pub inner: F,
        }

        impl<F: Future> Future for PanicCheckFuture<F> {
            type Output = F::Output;

            fn poll(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                if crate::panics::has_panicked() {
                    std::task::Poll::Pending
                } else {
                    self.project().inner.poll(cx)
                }
            }
        }

        /// A combiend `Weak` and `RefCell` that facilities upgrading and borrowing as a shared
        /// operation
        #[must_use]
        pub struct DeferredCtx<T: Component> {
            /// The `Weak<RefCell<T>>` in question
            inner: Weak<RefCell<State<T>>>,
        }

        // We put a bound on `'p` so that users are not able to store the upgraded reference (unless
        // they want to use ouroboros themself to store it alongside the weak).
        #[ouroboros::self_referencing]
        struct DeferredRefInner<'p, T: Component> {
            rc: Rc<RefCell<State<T>>>,
            lifetime: PhantomData<&'p ()>,
            #[borrows(rc)]
            #[covariant]
            reference: RefMut<'this, State<T>>,
        }

        /// a `RefMut` that also holds a `Rc`.
        /// See the `DeferredCtx::borrow_mut` on drop semantics and safety
        #[cfg_attr(feature = "nightly", must_not_suspend)]
        #[must_use]
        pub struct DeferredRef<'p, T: Component>(DeferredRefInner<'p, T>);

        impl<T: Component> DeferredCtx<T> {
            /// Borrow this `Weak<RefCell<...>>`, this will create a `Rc` for as long as the borrow is
            /// active. Returns `None` if the component was dropped. Its recommended to use the
            /// following construct to safely cancel async tasks:
            /// ```ignore
            /// let Some(mut borrow) = ctx.borrow_mut() else {return;};
            /// // ...
            /// drop(borrow);
            /// foo().await;
            /// let Some(mut borrow) = ctx.borrow_mut() else {return;};
            /// // ...
            /// ```
            ///
            /// # Reactivity
            /// Calling this function clears the internal reactive flags (which is safe as long as the
            /// borrow safety rules below are followed).
            /// Once this value is dropped it will trigger a reactive update for any changed fields.
            ///
            /// # Borrow Safety
            /// The framework guarantees that it will never hold a borrow between event calls.
            /// This means the only source of panics is if you are holding a borrow when you yield to
            /// the event loop, i.e you should *NOT* hold this value across `.await` points.
            /// framework will regularly borrow the state on any registered event handler trigger, for
            /// example a user clicking a button.
            ///
            /// Keeping this type across an `.await` point or otherwise yielding control to the event
            /// loop while the borrow is active could also lead to reactivity failrues and desyncs.
            ///
            /// ## Nightly
            /// The nightly feature flag enables a lint to detect this misuse.
            /// See the [Features]() chapther for details on how to set it up (it requires a bit more
            /// setup than just turning on the feature flag).
            #[must_use]
            pub fn borrow_mut(&self) -> Option<DeferredRef<'_, T>> {
                let rc = self.inner.upgrade()?;
                let borrow = DeferredRefInner::try_new(rc, PhantomData, |rc| rc.try_borrow_mut());

                let Ok(mut borrow) = borrow else {
                    log_or_panic!(
                        "Deferred state borrowed while already borrowed. This might happen due to holding it across a yield point"
                    );
                    return None;
                };

                borrow.with_reference_mut(|ctx| ctx.clear());
                Some(DeferredRef(borrow))
            }
        }

        impl<T: Component> Deref for DeferredRef<'_, T> {
            type Target = State<T>;

            #[inline]
            fn deref(&self) -> &Self::Target {
                self.0.borrow_reference()
            }
        }
        impl<T: Component> DerefMut for DeferredRef<'_, T> {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.0.with_reference_mut(|cell| &mut **cell)
            }
        }

        impl<T: Component> Drop for DeferredRef<'_, T> {
            fn drop(&mut self) {
                self.0.with_reference_mut(|ctx| {
                    ctx.update();
                });
            }
        }
    }
}
