//! A full css stylesheet

use crate::css::property::RuleBody;
use crate::css::selectors::IntoSelectorList;

/// A css stylesheet
#[must_use]
pub struct StyleSheet {
    /// Raw sections of css
    pub(crate) sections: Vec<String>,
}

impl Default for StyleSheet {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleSheet {
    /// Create a new stylesheet
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Add a raw section to the css
    pub fn raw(mut self, raw: impl Into<String>) -> Self {
        self.sections.push(raw.into());
        self
    }

    /// Add a rule to the stylesheet
    pub fn rule(mut self, selector: impl IntoSelectorList, body: RuleBody) -> Self {
        let selector = selector.into_list().into_css();
        let body = body.into_css();

        let section = format!("{selector}{{{body}}}");
        self.sections.push(section);

        self
    }

    /// Convert this to css
    #[doc(hidden)]
    #[must_use]
    pub fn to_css(self) -> String {
        self.sections.join("")
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        fn raw(text: String) {
            let style = StyleSheet::new().raw(&text);
            assert_eq!(style.to_css(), text);
        }
    }
}
