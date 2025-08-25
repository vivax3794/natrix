//! Reactivity system for tracking dependencies and updates.

pub mod mount;
pub mod render_callbacks;
pub mod signal;
pub mod state;
pub(crate) mod statics;

pub use state::{EventCtx, RenderCtx, State};

/// for keeping specific objects alive in memory such as `Closure` and `Rc`
pub(crate) type KeepAlive = Box<dyn std::any::Any>;
