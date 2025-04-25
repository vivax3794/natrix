//! utilities for writing unit tests on wasm
#![cfg(feature = "test_utils")]
#![expect(clippy::expect_used, reason = "tests only")]

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
/// # Panics
/// If the js is in a invalid state or the element is not found
pub fn mount_test<C: Component>(component: C) {
    setup();
    mount_at(component, MOUNT_POINT).expect("Failed to mount");
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
#[expect(clippy::panic, reason = "tests only")]
pub fn get(id: &'static str) -> HtmlElement {
    let document = get_document();

    document
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("Id {id} not found"))
        .dyn_ref::<HtmlElement>()
        .expect("Target Node wasnt a html element")
        .clone()
}
