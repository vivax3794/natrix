//! Reactivity system for tracking dependencies and updates.

pub mod callbacks;
pub mod component;
pub mod render_callbacks;
pub mod signal;
pub mod state;

// Re-export the public items
pub use callbacks::EventHandler;
pub use component::{Component, NonReactive};
pub use state::{EventToken, State};
