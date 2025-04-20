#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]
#![forbid(
    clippy::todo,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::unreachable,
    clippy::indexing_slicing,
    unsafe_code
)]
#![deny(
    clippy::dbg_macro,
    clippy::expect_used,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    clippy::arithmetic_side_effects
)]
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::pedantic,
    unfulfilled_lint_expectations
)]
#![allow(clippy::type_complexity, reason = "Fn trait objects get complex.")]
#![allow(
    private_interfaces,
    private_bounds,
    reason = "Our api design does this on purpose"
)]
#![cfg_attr(feature = "nightly", feature(must_not_suspend))]
#![cfg_attr(feature = "nightly", warn(must_not_suspend))]
#![cfg_attr(feature = "nightly", feature(associated_type_defaults))]
#![cfg_attr(nightly, feature(min_specialization))]

pub mod async_utils;
pub mod callbacks;
pub mod component;
pub mod element;
pub mod events;
pub mod html_elements;
pub mod list;
mod render_callbacks;
mod signal;
pub mod state;
pub mod test_utils;
mod type_macros;
mod utils;

thread_local! {
    /// A lazy initlized reference to the js document.
    static DOCUMENT: web_sys::Document = {
        #[expect(
            clippy::expect_used,
            reason = "A web framework cant do much without access to the document"
        )]
        web_sys::window()
            .expect("Window object not found")
            .document()
            .expect("Document object not found")
    };

    /// A lazy initlized reference to the js window.
    static WINDOW: web_sys::Window = {
        #[expect(
            clippy::expect_used,
            reason = "A web framework cant do much without access to the window"
        )]
        web_sys::window()
            .expect("Window object not found")
    };

}

/// Get the globally acquired document
///
/// This is cached so we dont need the slowdown of the js interop and `Result` handling for every
/// use of document.
pub(crate) fn get_document() -> web_sys::Document {
    DOCUMENT.with(Clone::clone)
}

/// Get the globally acquired window
///
/// This is cached so we dont need the slowdown of the js interop and `Result` handling for every
/// use of window.
pub(crate) fn get_window() -> web_sys::Window {
    WINDOW.with(Clone::clone)
}

/// Panic handling
#[cfg(feature = "panic_hook")]
mod panics {
    /// Mark that a panic has happened
    static PANIC_HAPPENED: std::sync::Once = std::sync::Once::new();

    /// Is the panic hook set?
    pub(crate) fn has_panicked() -> bool {
        let result = PANIC_HAPPENED.is_completed();
        #[cfg(debug_assertions)]
        if result {
            web_sys::console::warn_1(
                &"
Access to framework state was attempted after a panic.
Continuing to execute rust code after panic may cause undefined behavior.
If you which to allow execution after a panic (not recommended) you can disable the `panic_hook` feature of `natrix`.
"
                .trim()
                .into(),
            );
        }
        result
    }

    /// Set the panic hook to mark that a panic has happened
    pub fn set_panic_hook() {
        std::panic::set_hook(Box::new(|info| {
            PANIC_HAPPENED.call_once(|| {
                // This is a no-op, we just want to mark that a panic has happened
            });

            #[cfg(debug_assertions)]
            {
                let panic_message = info.to_string();
                web_sys::console::error_1(&panic_message.into());
            }
        }));
    }
}

#[cfg(feature = "panic_hook")]
pub use panics::set_panic_hook;

/// Returns if a panic has happened
///
/// (or is a noop when the `panic_hook` feature is not enabled)
macro_rules! return_if_panic {
    ($val:expr) => {
        #[cfg(feature = "panic_hook")]
        if $crate::panics::has_panicked() {
            return $val;
        }
    };
    () => {
        #[cfg(feature = "panic_hook")]
        if $crate::panics::has_panicked() {
            return;
        }
    };
}
pub(crate) use return_if_panic;

/// Public export of everything.
pub mod prelude {
    pub use natrix_macros::Component;

    pub use super::component::{Component, NoMessages, SubComponent};
    pub use super::element::Element;
    pub use super::state::{E, R};
    pub use super::{events, html_elements as e};
}
pub use component::{Component, NoMessages, SubComponent, mount};
pub use element::Element;
pub use list::List;
pub use natrix_macros;
#[cfg(feature = "scoped_css")]
pub use natrix_macros::scoped_css;
#[cfg(feature = "inline_css")]
pub use natrix_macros::style;
pub use natrix_macros::{Component, data, global_css};
pub use state::{RenderCtx, State};

/// Public exports of internal data structures for `natrix_macros` to use in generated code.
#[doc(hidden)]
pub mod macro_ref {
    pub use super::component::ComponentBase;
    pub use super::signal::{Signal, SignalMethods, SignalState};
    pub use super::state::{ComponentData, E, Guard, State};
}
