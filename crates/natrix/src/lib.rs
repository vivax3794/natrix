#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]
#![cfg_attr(feature = "nightly", feature(must_not_suspend))]
#![cfg_attr(feature = "nightly", warn(must_not_suspend))]
#![cfg_attr(feature = "nightly", feature(associated_type_defaults))]
#![cfg_attr(nightly, feature(cold_path))]
#![cfg_attr(not(feature = "_internal_no_ssg"), forbid(unsafe_code))]

pub mod async_utils;
pub mod css;
pub mod dom;
mod error_handling;
pub mod panics;
pub mod reactivity;
pub mod test_utils;
mod type_macros;

pub use wasm_bindgen::intern;

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

/// Commonly used types and traits.
pub mod prelude {
    pub use natrix_macros::Component;

    pub use super::css::selectors::{
        Class,
        Id,
        IntoComplexSelector,
        IntoCompoundSelector,
        IntoFinalizedSelector,
    };
    pub use super::dom::{Element, events, html_elements as e};
    pub use super::reactivity::component::{Component, NoMessages, SubComponent};
    pub use super::reactivity::state::{E, R};
}

pub use dom::Element;
pub use dom::list::List;
pub use natrix_macros::{Component, asset, data, format_elements};
pub use reactivity::component::{Component, NoMessages, SubComponent, mount};
pub use reactivity::state::{RenderCtx, State};

/// Public exports of internal data structures for `natrix_macros` (and `macro_rules`) to use in generated code.
#[doc(hidden)]
pub mod macro_ref {
    #[cfg(feature = "_internal_collect_css")]
    pub use inventory;
    pub use {const_base, const_sha1, log};

    pub use super::css;
    pub use super::dom::element::Element;
    pub use super::reactivity::component::ComponentBase;
    pub use super::reactivity::signal::{Signal, SignalMethods, SignalState};
    pub use super::reactivity::state::{ComponentData, E, Guard, State};
}
