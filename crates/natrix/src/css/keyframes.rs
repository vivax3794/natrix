//! Implement css keyframes

use super::values::ToCssValue;

/// The name of a keyframe
pub struct KeyFrame(pub &'static str);

// TODO: Finish this implementation
// SPEC: Keyframes cant contain `!important`
// (which we currently dont support setting on properties to begin with)
// SPEC: Not all properties can be animated in keyframes

impl ToCssValue for KeyFrame {
    fn to_css(self) -> String {
        super::as_css_identifier(self.0)
    }
}

/// A keyframe definition
#[derive(Clone, Default)]
#[must_use]
pub struct KeyframeDefinition {}

impl KeyframeDefinition {
    /// Create a new empty keyframe definition
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert this into css
    #[doc(hidden)]
    #[must_use]
    pub fn to_css(self, name: &'static str) -> String {
        let inner = "";
        format!("@keyframes {} {{{inner}}}", super::as_css_identifier(name))
    }
}

/// Define and register a `@keyframe` to go in the bundler.
///
/// Any code in here wont be included in the final wasm build.
/// And will be run at compile time.
/// This macro must not be called from within a function.
#[macro_export]
macro_rules! register_keyframe {
    ($vis:vis const $name:ident = $def:expr;) => {
        $vis const $name: $crate::macro_ref::css::keyframes::KeyFrame = $crate::macro_ref::css::keyframes::KeyFrame($crate::unique_str!());
        $crate::register_raw_css!({
            use $crate::macro_ref::css::prelude::*;
            use $crate::macro_ref::css::keyframes::KeyframeDefinition;
            let result: $crate::macro_ref::css::keyframes::keyframeDefinition = $def;
            result.to_css($name)
        });
    };
}
