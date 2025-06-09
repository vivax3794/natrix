//! utilities for writing unit tests on wasm
#![cfg(feature = "test_utils")]
#![expect(clippy::expect_used, reason = "tests only")]

use std::cell::Cell;

use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::get_document;
use crate::prelude::Component;
use crate::reactivity::component::render_component;
use crate::reactivity::state::KeepAlive;

/// The parent of the testing env
const MOUNT_PARENT: &str = "__TESTING_PARENT";
/// The var where you should mount your component
/// This is auto created and cleaned up by `setup`
pub const MOUNT_POINT: &str = "__TESTING_MOUNT_POINT";

thread_local! {
     static CURRENT_COMP: Cell<KeepAlive>  = Cell::new(Box::new(()));
}

/// Has a logger be initlized?
#[cfg(feature = "console_log")]
static LOGGER_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Mount a component at the test location (creating/resetting it if needed)
/// # Panics
/// If the js is in a invalid state or the element is not found
pub fn mount_test<C: Component>(component: C) {
    #[cfg(feature = "console_log")]
    {
        let was_logger_active = LOGGER_ACTIVE.fetch_or(true, std::sync::atomic::Ordering::Relaxed);
        if !was_logger_active {
            console_log::init_with_level(log::Level::Trace).expect("Failed to setup logging");
        }
    }

    setup();

    log::debug!("Mounting test component {}", std::any::type_name::<C>());
    let result = render_component(component, MOUNT_POINT).expect("Failed to mount");
    CURRENT_COMP.with(|cell| cell.set(Box::new(result)));
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
        log::trace!("Removed old test tree");
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

    log::trace!("Setup test target");
}

/// Get a html element based on id
///
/// # Panics
/// If js is in a invalid state or the element isnt found
#[must_use]
#[expect(clippy::panic, reason = "tests only")]
pub fn get(id: impl Into<&'static str>) -> HtmlElement {
    let id = id.into();

    let document = get_document();

    document
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("Id {id} not found"))
        .dyn_ref::<HtmlElement>()
        .expect("Target Node wasnt a html element")
        .clone()
}
