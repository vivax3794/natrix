#![doc = include_str!("../../README.md")]
#![deny(
    unsafe_code,
    clippy::todo,
    clippy::dbg_macro,
    clippy::unreachable,
    clippy::unwrap_used,
    unfulfilled_lint_expectations,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason
)]
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::pedantic,
    clippy::expect_used,
    clippy::unreachable
)]
#![allow(clippy::type_complexity, reason = "Fn trait objects get complex.")]
#![allow(
    private_interfaces,
    private_bounds,
    reason = "Our api design does this on purpose"
)]
#![cfg_attr(feature = "nightly", feature(must_not_suspend))]
#![cfg_attr(nightly, feature(min_specialization))]

use std::cell::OnceCell;

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
    static DOCUMENT: OnceCell<web_sys::Document> = const { OnceCell::new() };
}

/// Get the globaly aquired document
///
/// This is cached so we dont need the slowdown of the js interop and `Result` handling for every
/// use of document.
pub(crate) fn get_document() -> web_sys::Document {
    DOCUMENT.with(|doc_cell| {
        doc_cell
            .get_or_init(|| {
                #[expect(
                    clippy::expect_used,
                    reason = "A web framework cant do much without access to the document"
                )]
                web_sys::window()
                    .expect("Window object not found")
                    .document()
                    .expect("Document object not found")
            })
            .clone()
    })
}

/// Public export of everything.
pub mod prelude {
    pub use natrix_macros::Component;

    pub use super::component::{C, Component, mount_component};
    pub use super::element::Element;
    pub use super::state::{R, S};
    pub use super::{events, guard_option, guard_result, html_elements as e};
}

#[cfg(feature = "test_utils")]
#[expect(clippy::unwrap_used, reason = "tests only")]
/// utilities for writting unit tests on wasm
pub mod test_utils {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlElement;

    /// The parent of the testing env
    const MOUNT_PARENT: &str = "__TESTING_PARENT";
    /// The var where you should mount your component
    /// This is auto created and cleaned up by `setup`
    pub const MOUNT_POINT: &str = "__TESTING_MOUNT_POINT";

    /// Setup `MOUNT_POINt` as a valid mount location
    ///
    /// # Panics
    /// if the js is in a invalid state.
    pub fn setup() {
        let document = web_sys::window().unwrap().document().unwrap();

        if let Some(element) = document.get_element_by_id(MOUNT_PARENT) {
            element.remove();
        }

        let parent = document.create_element("div").unwrap();
        parent.set_id(MOUNT_PARENT);

        let mount = document.create_element("div").unwrap();
        mount.set_id(MOUNT_POINT);

        parent.append_child(&mount).unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
    }

    /// Get a html element based on id
    ///
    /// # Panics
    /// If js is in a invalid state or the element isnt found
    #[must_use]
    pub fn get(id: &'static str) -> HtmlElement {
        let document = web_sys::window().unwrap().document().unwrap();

        document
            .get_element_by_id(id)
            .unwrap()
            .dyn_ref::<HtmlElement>()
            .unwrap()
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

// Cargo-mutants cant run `wasm-pack`, this is a workaround to ensure its tests get included
#[cfg(mutants)]
#[test]
fn wasm_pack_test() {
    use std::process::Command;
    let output = Command::new("wasm-pack")
        .args(["test", "--headless", "--chrome"])
        .output()
        .expect("Failed to run wasm-pack test");

    assert!(output.status.success(), "wasm-pack test failed!");
}
