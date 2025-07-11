//! Reactivity system for tracking dependencies and updates.

// TODO: Global state stores
// TODO: A router, using global state stores
// TODO: Allow setting `<head>` content reactively

pub mod mount;
pub mod render_callbacks;
pub mod signal;
pub mod state;
mod statics;

pub use state::{Ctx, EventToken, RenderCtx, State};

/// for keeping specific objects alive in memory such as `Closure` and `Rc`
pub(crate) type KeepAlive = Box<dyn std::any::Any>;
