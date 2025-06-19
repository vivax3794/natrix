//! Create css styles

// NOTE:
// This module does not generally need to be efficient, it just needs to be ergonomic.
// The code in this module is intended to be used at bundling time, and hence will not be included
// in production applications.

// TODO: Implmenet all of css
// TODO: Allow composing css rules similar to css-modules `composes`

pub mod property;
pub mod selectors;
pub mod stylesheet;
pub mod values;

pub use selectors::{PseudoClass, PseudoClassNested};
pub use stylesheet::StyleSheet;
pub use values::Color;

/// Struct to let `inventory` collect css from all across the dep graph
#[doc(hidden)]
#[cfg(feature = "_internal_collect_css")]
pub struct CssEmit(pub fn() -> String);

#[cfg(feature = "_internal_collect_css")]
inventory::collect!(CssEmit);

#[cfg(feature = "_internal_no_ssg")]
#[expect(
    unsafe_code,
    reason = "This is required for inventory to work, it is not included in production builds"
)]
unsafe extern "C" {
    fn __wasm_call_ctors();
}

cfg_if::cfg_if! {
    if #[cfg(feature = "_internal_collect_css")] {
        /// Register a css stylesheet to go in the bundler.
        /// Any code in here wont be included in the final wasm build.
        /// And will be run at compile time.
        ///
        /// This macro must not be called from within a function.
        #[macro_export]
        #[cfg(feature = "_internal_collect_css")]
        macro_rules! register_css {
            ($style:expr) => {
                $crate::macro_ref::inventory::submit!($crate::macro_ref::CssEmit(|| {
                    use $crate::css::prelude::*;
                    $crate::macro_ref::log::trace!(concat!("generating css for ", file!(), " ", line!()));
                    let sheet: $crate::macro_ref::StyleSheet = $style;
                    sheet.to_css()
                }));
            };
        }
    }
    else {
        /// Register a css stylesheet to go in the bundler.
        /// Any code in here wont be included in the final wasm build.
        /// And will be run at compile time.
        ///
        /// This macro must not be called from within a function.
        #[macro_export]
        #[cfg(not(feature = "_internal_collect_css"))]
        macro_rules! register_css {
            ($style:expr) => {
                const _: fn() -> $crate::macro_ref::StyleSheet = || {
                    use $crate::css::prelude::*;
                    $crate::macro_ref::log::warn!("Register css code called in non-collection mode");
                    $style
                };
            };
        }
    }
}

/// Css prelude
/// This is auto star imported in `register_css`
pub mod prelude {
    pub use super::PseudoClass::*;
    pub use super::PseudoClassNested::*;
    pub use super::StyleSheet;
    pub use super::selectors::{Direction, NthArgument};
    pub use crate::selector_list;
}

/// Do the css collection and either emit to STDOUT or inject into dom
/// Depending on the selected feature flags.
#[cfg(feature = "_internal_collect_css")]
pub(crate) fn css_collect() {
    log::info!("Collecting css");

    #[cfg(feature = "_internal_no_ssg")]
    #[expect(
        unsafe_code,
        reason = "This is required for inventory to work on wasm, it is not included in production builds"
    )]
    unsafe {
        log::trace!("Calling ctors");
        __wasm_call_ctors();
    }

    log::trace!("Collecting strings");
    let mut result = String::new();
    for emit in inventory::iter::<CssEmit> {
        result.push_str(&(emit.0)());
    }

    #[cfg(feature = "_internal_no_ssg")]
    css_runtime(&result);

    #[cfg(feature = "_internal_bundle")]
    css_emit(&result);
}

/// Inject the css into the dom at runtime
#[cfg(feature = "_internal_no_ssg")]
#[expect(
    clippy::expect_used,
    reason = "This happens early, and is meant for dev mode only"
)]
fn css_runtime(css_string: &str) {
    log::debug!("Injecting css into document");
    let document = crate::get_document();
    let style = document
        .create_element("style")
        .expect("Failed to create style element");

    style.set_inner_html(css_string);

    let body = document.body().expect("No body found");
    body.append_child(&style)
        .expect("Failed to append style element");
}

/// Print out the css to provide it to the bundler
///
/// # Design
/// Why not just `println!` in `do_css_collect`? because we might refactor this to use something a
/// bit more structured later (json, files, whatever)
#[cfg(feature = "_internal_bundle")]
fn css_emit(css_string: &str) {
    log::debug!("Emitting css to bundler");
    println!("{css_string}");
}

/// Create a unique string
///
/// This is a hash of the filename + line number + column (computed at compile time)
///
/// This is used internally by the `class` and `id` macros
/// ```rust
/// # use natrix::prelude::*;
/// use natrix::class;
///
/// const MY_CLASS: Class = class!(); // <-- uses `unique_str`
/// ```
#[macro_export]
macro_rules! unique_str {
    () => {{
        const RAW: &str = concat!(file!(), "-", line!(), "-", column!());
        const HASHED: [u8; 20] = $crate::macro_ref::const_sha1::sha1(RAW.as_bytes()).as_bytes();
        const ENCODED: &str = $crate::macro_ref::const_base::encode_as_str!(
            &HASHED,
            $crate::macro_ref::const_base::Config::B64_URL_SAFE.end_padding(false),
        );

        ENCODED
    }};
}

/// Check if a string is valid css
#[cfg(all(test, not(target_arch = "wasm32")))]
#[expect(clippy::panic, clippy::expect_used, reason = "This is meant for tests")]
fn assert_valid_css(string: &str) {
    let warnings = std::sync::Arc::default();
    let result = lightningcss::stylesheet::StyleSheet::parse(
        string,
        lightningcss::stylesheet::ParserOptions {
            warnings: Some(std::sync::Arc::clone(&warnings)),
            error_recovery: false,
            ..Default::default()
        },
    );

    match result {
        Err(error) => {
            panic!("The following code was not valid css\n{string}\nerror: {error}");
        }

        Ok(stylesheet) => {
            let warnings = warnings.read().expect("Failed to get lock");
            if !warnings.is_empty() {
                for warning in warnings.iter() {
                    eprintln!("{warning}");
                }
                panic!("The following code produced warnings\n{string}");
            }

            // I cant be arsed to write a visitor for this
            let debug_reps = format!("{stylesheet:?}");

            let invalid = debug_reps.contains("Unparsed") || debug_reps.contains("Unknown");
            assert!(
                !invalid,
                "Found indications of invalid css\n{string}\n{stylesheet:?}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn unique_is_unique() {
        assert_ne!(unique_str!(), unique_str!());
    }
}
