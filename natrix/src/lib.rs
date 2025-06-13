#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]
#![cfg_attr(feature = "nightly", feature(must_not_suspend))]
#![cfg_attr(feature = "nightly", warn(must_not_suspend))]
#![cfg_attr(feature = "nightly", feature(associated_type_defaults))]
#![cfg_attr(nightly, feature(cold_path))]
#![cfg_attr(not(feature = "_internal_runtime_css"), forbid(unsafe_code))]

pub mod async_utils;
pub mod css;
pub mod dom;
mod error_handling;
pub mod panics;
pub mod reactivity;
pub mod test_utils;
mod type_macros;

use std::ops::ControlFlow;

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

/// Public export of everything.
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
pub use natrix_macros::{Component, asset, data};
pub use reactivity::component::{Component, NoMessages, SubComponent, mount};
pub use reactivity::state::{RenderCtx, State};

/// Setup the various runtime systems for natrix.
/// This installs the panic hook, initialises loggers if enabled.
/// if in bundler-mode it will also be the entrypoint that extracts the css to a static file.
/// Or if in non-ssg mode (i.e dev or explicitly requested in config) be the one that mounts the
/// styles to the dom.
///
/// If this returns `ControlFlow::Break` you should exit immediately.
/// As this indicated a bundle-time run of the application and no component should be attempted
/// mounted.
///
/// Its is very important no output to stdout is written before or after this function.
///
/// This function is automatically called by mount.
pub fn setup_runtime() -> ControlFlow<()> {
    crate::panics::set_panic_hook();
    #[cfg(feature = "console_log")]
    if cfg!(target_arch = "wasm32") {
        if let Err(err) = console_log::init_with_level(log::Level::Trace) {
            crate::error_handling::debug_panic!("Failed to create logger: {err}");
        }
    }
    #[cfg(feature = "_internal_extract_css")]
    if let Err(err) = simple_logger::init_with_level(log::Level::Trace) {
        eprintln!("Failed to setup logger {err}");
    }
    log::info!("Logging initialized");
    #[cfg(feature = "_internal_collect_css")]
    crate::css::css_collect();

    if cfg!(feature = "_internal_extract_css") {
        log::info!("Css extract mode, aboring mount.");
        return ControlFlow::Break(());
    }
    ControlFlow::Continue(())
}

/// Public exports of internal data structures for `natrix_macros` (and `macro_rules`) to use in generated code.
#[doc(hidden)]
pub mod macro_ref {
    #[cfg(feature = "_internal_collect_css")]
    pub use inventory;
    pub use {const_base, const_sha1, log};

    #[cfg(feature = "_internal_collect_css")]
    pub use super::css::CssEmit;
    pub use super::css::stylesheet::StyleSheet;
    pub use super::reactivity::component::ComponentBase;
    pub use super::reactivity::signal::{Signal, SignalMethods, SignalState};
    pub use super::reactivity::state::{ComponentData, E, Guard, State};
}
