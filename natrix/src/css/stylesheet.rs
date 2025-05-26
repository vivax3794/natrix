//! A full css stylesheet

/// A css stylesheet
#[must_use]
pub struct StyleSheet {
    /// Raw sections of css
    pub(crate) raw_sections: Vec<String>,
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
            raw_sections: Vec::new(),
        }
    }

    /// Add a raw section to the css
    pub fn raw(mut self, raw: impl Into<String>) -> Self {
        self.raw_sections.push(raw.into());
        self
    }

    /// Convert this to css
    #[doc(hidden)]
    #[must_use]
    pub fn to_css(self) -> String {
        self.raw_sections.join("\n")
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
