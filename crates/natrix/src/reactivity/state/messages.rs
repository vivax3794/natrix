//! Message handling

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use smallvec::SmallVec;

use super::{EventToken, State};
use crate::error_handling::log_or_panic;
use crate::reactivity::component::Component;

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
pub(super) type DeferredMessageQueue<C> = RefCell<SmallVec<[InternalMessage<C>; 1]>>;

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
pub(super) type EmitMessageSender<Msg> = Box<dyn Fn(Vec<Msg>)>;

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

            direct.track_changes(|ctx| {
                for message in messages {
                    ctx.handle_message(message);
                }
            });
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

impl<T: Component> State<T> {
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
    pub(super) fn drain_message_queue(&mut self) {
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
}
