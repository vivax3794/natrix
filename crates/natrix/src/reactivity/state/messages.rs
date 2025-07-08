//! Message handling

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use smallvec::SmallVec;

use super::{Ctx, EventToken};
use crate::error_handling::log_or_panic;
use crate::reactivity::State;

/// A message on the internal messing system
#[derive(Debug)]
pub(crate) enum InternalMessage {}

/// The queue of messages sent to a component while it was borrowed
pub(super) type DeferredMessageQueue = RefCell<SmallVec<[InternalMessage; 1]>>;

/// Send messages to a component which are executed right away if component is not borrowed.
/// If the component is borrowed its assumed we are in a recursive event context and the messages
/// are appended to a queue.
/// The component should check this queue on its next update call.
pub(crate) struct EagerMessageSender<C: State> {
    /// A direct reference to the state
    direct: Weak<RefCell<Ctx<C>>>,
    /// A reference to the message queue
    deferred: Weak<DeferredMessageQueue>,
}

impl<C: State> Clone for EagerMessageSender<C> {
    fn clone(&self) -> Self {
        Self {
            direct: Weak::clone(&self.direct),
            deferred: Weak::clone(&self.deferred),
        }
    }
}

impl<C: State> EagerMessageSender<C> {
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
    pub(crate) fn send(&self, message: InternalMessage) -> Option<()> {
        self.send_batched(std::iter::once(message))
    }

    /// Send multiple messages at once.
    /// This method avoids the overhead of multiple `RefCell` checks and reactive updates.
    pub(crate) fn send_batched(
        &self,
        messages: impl IntoIterator<Item = InternalMessage>,
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

impl<T: State> Ctx<T> {
    /// Handle a internal message
    ///
    /// This does not trigger clean reactive tracking or updates of the action
    /// This is to allow batching messages handling.
    /// Calling `.clear` and `.update` is meant for the caller
    fn handle_message(&mut self, message: InternalMessage) {
        log::debug!("Handling message {message:?}");
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
}
