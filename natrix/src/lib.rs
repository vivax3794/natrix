#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]
#![cfg_attr(feature = "nightly", feature(must_not_suspend))]
#![cfg_attr(feature = "nightly", warn(must_not_suspend))]
#![cfg_attr(feature = "nightly", feature(associated_type_defaults))]
#![cfg_attr(nightly, feature(cold_path))]

pub mod async_utils;
pub mod css;
pub mod dom;
pub mod reactivity;
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
pub mod panics {
    /// Mark that a panic has happened
    static PANIC_HAPPENED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);

    /// Has a panic occurred
    /// This is only needed for you to call if you are using custom callbacks passed to js.
    /// All natrix event handlers already check this.
    /// And all uses of `ctx.use_async` uses some magic to insert a check to this *after every*
    /// await.
    pub fn has_panicked() -> bool {
        let result = PANIC_HAPPENED.load(std::sync::atomic::Ordering::Relaxed);
        #[cfg(console_log)]
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
        #[cfg(target_arch = "wasm32")]
        #[allow(clippy::allow_attributes, reason = "only applies sometimes")]
        #[allow(unused_variables, reason = "Only used when console_log is enabled")]
        std::panic::set_hook(Box::new(|info| {
            PANIC_HAPPENED.store(true, std::sync::atomic::Ordering::Relaxed);

            #[cfg(console_log)]
            {
                let panic_message = info.to_string();
                web_sys::console::error_1(&panic_message.into());
            }
        }));
    }
}

/// Returns if a panic has happened
///
/// (or is a noop when the `panic_hook` feature is not enabled)
macro_rules! return_if_panic {
    ($val:expr) => {
        if $crate::panics::has_panicked() {
            return $val;
        }
    };
    () => {
        if $crate::panics::has_panicked() {
            return;
        }
    };
}
pub(crate) use return_if_panic;

/// Public export of everything.
pub mod prelude {
    pub use natrix_macros::Component;

    pub use super::dom::{Element, events, html_elements as e};
    pub use super::reactivity::component::{Component, NoMessages, SubComponent};
    pub use super::reactivity::state::{E, R};
}
pub use dom::Element;
pub use dom::list::List;
pub use natrix_macros::{Component, asset, data};
pub use reactivity::component::{Component, NoMessages, SubComponent, mount};
pub use reactivity::state::{RenderCtx, State};

/// Public exports of internal data structures for `natrix_macros` to use in generated code.
#[doc(hidden)]
pub mod macro_ref {
    pub use super::reactivity::component::ComponentBase;
    pub use super::reactivity::signal::{Signal, SignalMethods, SignalState};
    pub use super::reactivity::state::{ComponentData, E, Guard, State};
}
