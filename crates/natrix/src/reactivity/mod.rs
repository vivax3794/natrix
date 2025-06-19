//! Reactivity system for tracking dependencies and updates.

// TODO: Global state stores
// TODO: A router, using global state stores
// TODO: Allow setting `<head>` content reactively

pub mod component;
pub mod non_reactive;
pub mod render_callbacks;
pub mod signal;
pub mod state;

pub use component::Component;
pub use non_reactive::NonReactive;
pub use state::{EventToken, State};

/// for keeping specific objects alive in memory such as `Closure` and `Rc`
pub(crate) type KeepAlive = Box<dyn std::any::Any>;
