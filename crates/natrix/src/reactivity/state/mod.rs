//! Types for handling the application state

#[cfg(feature = "async")]
mod async_state;
mod core;
mod data_manager;
pub mod guards;
mod hook_manager;
mod watch;

#[cfg(feature = "async")]
pub use self::async_state::{AsyncCtx, AsyncCtxHandle};
pub(crate) use self::core::InnerCtx;
pub use self::core::{EventCtx, RenderCtx};
pub(crate) use self::data_manager::HookDepListHolder;
pub use self::data_manager::State;
pub(crate) use self::hook_manager::{HookKey, IterSignalList, SignalDepList};
