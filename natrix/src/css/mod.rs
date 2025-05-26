//! Create css styles

pub mod stylesheet;
pub mod values;

pub use stylesheet::StyleSheet;
pub use values::Color;

/// Struct to let `inventory` collect css from all across the dep graph
#[doc(hidden)]
#[cfg(feature = "_internal_collect_css")]
pub struct CssEmit(pub fn() -> String);

#[cfg(feature = "_internal_collect_css")]
inventory::collect!(CssEmit);

#[cfg(feature = "_internal_runtime_css")]
#[expect(
    unsafe_code,
    reason = "This is required for inventory to work, it is not included in production builds"
)]
unsafe extern "C" {
    fn __wasm_call_ctors();
}

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
            let sheet: $crate::macro_ref::StyleSheet = $style;
            sheet.to_css()
        }));
    };
}

/// Register a css stylesheet to go in the bundler.
/// Any code in here wont be included in the final wasm build.
/// And will be run at compile time.
///
/// This macro must not be called from within a function.
#[macro_export]
#[cfg(not(feature = "_internal_collect_css"))]
macro_rules! register_css {
    ($style:expr) => {
        const _: fn() -> $crate::macro_ref::StyleSheet = || $style;
    };
}

/// Do the css collection and either emit to STDOUT or inject into dom
/// Depending on the selected feature flags.
#[cfg(feature = "_internal_collect_css")]
pub(crate) fn css_collect() {
    #[cfg(feature = "_internal_runtime_css")]
    #[expect(
        unsafe_code,
        reason = "This is required for inventory to work on wasm, it is not included in production builds"
    )]
    unsafe {
        __wasm_call_ctors();
    }

    let mut result = String::new();
    for emit in inventory::iter::<CssEmit> {
        result.push_str(&(emit.0)());
    }

    #[cfg(feature = "_internal_runtime_css")]
    css_runtime(&result);

    #[cfg(feature = "_internal_extract_css")]
    css_emit(&result);
}

/// Inject the css into the dom at runtime
#[cfg(feature = "_internal_runtime_css")]
#[expect(
    clippy::expect_used,
    reason = "This happens early, and is meant for dev mode only"
)]
fn css_runtime(css_string: &str) {
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
#[cfg(feature = "_internal_extract_css")]
fn css_emit(css_string: &str) {
    println!("{css_string}");
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
