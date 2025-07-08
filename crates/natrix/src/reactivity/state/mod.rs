//! Types for handling the component state

// TODO: Reacting to reactive changes
// NOTE: This we removed as a feature because being able to emit messages during the update cycle
// led to inconsistent code.

#[cfg(feature = "async")]
mod async_state;
mod core;
mod data_manager;
mod guards;
mod hook_manager;
pub(crate) mod messages;
mod watch;

#[cfg(feature = "async")]
pub use self::async_state::DeferredCtx;
pub use self::core::{Ctx, RenderCtx};
pub use self::data_manager::State;
#[doc(hidden)]
pub use self::hook_manager::HookKey;

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
