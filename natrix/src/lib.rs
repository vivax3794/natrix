#![doc = include_str!("../../README.md")]
#![deny(unsafe_code, clippy::todo)]
#![warn(missing_docs, clippy::missing_docs_in_private_items, clippy::pedantic)]
#![allow(private_interfaces, private_bounds, clippy::type_complexity)]
#![cfg_attr(feature = "nightly", feature(must_not_suspend))]

use std::cell::OnceCell;

#[cfg(feature = "async")]
pub mod async_utils;
pub mod callbacks;
pub mod component;
pub mod element;
pub mod html_elements;
mod render_callbacks;
mod signal;
pub mod state;
mod type_macros;
mod utils;

thread_local! {
    /// A lazy initlized reference to the js document.
    static DOCUMENT: OnceCell<web_sys::Document> = const { OnceCell::new() };
}

/// Get the globaly aquired document
///
/// This is cached so we dont need the slowdown of the js interop and `Result` handling for every
/// use of document.
pub(crate) fn get_document() -> web_sys::Document {
    DOCUMENT.with(|doc_cell| {
        doc_cell
            .get_or_init(|| web_sys::window().unwrap().document().unwrap())
            .clone()
    })
}

/// Public export of everything.
pub mod prelude {
    pub use natrix_macros::Component;

    pub use super::callbacks::Event;
    pub use super::component::{Component, mount_component};
    pub use super::element::Element;
    pub use super::html_elements as e;
    pub use super::state::{ComponentData, S, State};
}

/// Public exports of internal data structures for `natrix_macros` to use in generated code.
#[doc(hidden)]
pub mod macro_ref {
    pub use super::component::ComponentBase;
    pub use super::signal::{Signal, SignalMethods};
    pub use super::state::ComponentData;
}
