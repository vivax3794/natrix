#![doc = include_str!("../../README.md")]
#![forbid(
    unsafe_code,
    clippy::todo,
    clippy::unreachable,
    clippy::unwrap_used,
    clippy::unreachable,
    clippy::indexing_slicing
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
#![cfg_attr(nightly, feature(min_specialization))]

#[cfg(feature = "async")]
pub mod async_utils;
pub mod callbacks;
pub mod component;
pub mod element;
pub mod events;
pub mod html_elements;
mod render_callbacks;
mod signal;
pub mod state;
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
}

/// Get the globaly aquired document
///
/// This is cached so we dont need the slowdown of the js interop and `Result` handling for every
/// use of document.
pub(crate) fn get_document() -> web_sys::Document {
    DOCUMENT.with(Clone::clone)
}

/// Public export of everything.
pub mod prelude {
    pub use natrix_macros::{Component, global_css, scoped_css};

    pub use super::component::{C, Component, mount};
    pub use super::element::Element;
    pub use super::state::{R, S};
    pub use super::{events, guard_option, guard_result, html_elements as e};
}

#[cfg(feature = "test_utils")]
#[expect(clippy::expect_used, reason = "tests only")]
/// utilities for writting unit tests on wasm
pub mod test_utils {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;

    use crate::component::mount_at;
    use crate::get_document;
    use crate::prelude::Component;

    /// The parent of the testing env
    const MOUNT_PARENT: &str = "__TESTING_PARENT";
    /// The var where you should mount your component
    /// This is auto created and cleaned up by `setup`
    pub const MOUNT_POINT: &str = "__TESTING_MOUNT_POINT";

    /// Mount a component at the test location (creating/resetting it if needed)
    pub fn mount_test<C: Component>(component: C) {
        setup();
        mount_at(component, MOUNT_POINT);
    }

    /// Setup `MOUNT_POINt` as a valid mount location
    ///
    /// # Panics
    /// if the js is in a invalid state.
    pub fn setup() {
        let document = web_sys::window()
            .expect("Failed to get window")
            .document()
            .expect("Failed to get document");

        if let Some(element) = document.get_element_by_id(MOUNT_PARENT) {
            element.remove();
        }

        let parent = document
            .create_element("div")
            .expect("Failed to create div");
        parent.set_id(MOUNT_PARENT);

        let mount = document
            .create_element("div")
            .expect("Failed to create div");
        mount.set_id(MOUNT_POINT);

        parent.append_child(&mount).expect("Failed to append child");
        document
            .body()
            .expect("Could not find <body>")
            .append_child(&parent)
            .expect("Failed to append child");
    }

    /// Get a html element based on id
    ///
    /// # Panics
    /// If js is in a invalid state or the element isnt found
    #[must_use]
    pub fn get(id: &'static str) -> HtmlElement {
        let document = get_document();

        document
            .get_element_by_id(id)
            .unwrap_or_else(|| panic!("Id {id} not found"))
            .dyn_ref::<HtmlElement>()
            .expect("Target Node wasnt a html element")
            .clone()
    }
}

/// Public exports of internal data structures for `natrix_macros` to use in generated code.
#[doc(hidden)]
pub mod macro_ref {
    pub use super::component::ComponentBase;
    pub use super::signal::{Signal, SignalMethods, SignalState};
    pub use super::state::{ComponentData, Guard, S};
}
