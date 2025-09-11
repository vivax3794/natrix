//! Implementation of the `Background` struct.

use crate::css::values::CssPropertyValue;
use crate::css::{IntoCss, values};

/// Represents a css background.
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/background>
#[derive(Debug, Clone)]
#[must_use]
pub struct Background {
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/background-attachment>
    attachment: String,
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/background-origin>
    origin: String,
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/background-clip>
    clip: String,
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/background-position>
    position: String,
}

impl Background {
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/background-attachment>
    pub fn attachment(mut self, value: impl CssPropertyValue<Kind = values::Attachment>) -> Self {
        self.attachment = value.into_css();
        self
    }

    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/background-origin>
    pub fn origin(mut self, value: impl CssPropertyValue<Kind = values::Attachment>) -> Self {
        self.origin = value.into_css();
        self
    }

    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/background-clip>
    pub fn clip(mut self, value: impl CssPropertyValue<Kind = values::Attachment>) -> Self {
        self.clip = value.into_css();
        self
    }

    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/background-position>
    pub fn position(mut self, value: impl CssPropertyValue<Kind = values::Attachment>) -> Self {
        self.position = value.into_css();
        self
    }
}

impl IntoCss for Background {
    fn into_css(self) -> String {
        format!(
            "{} {} {} {}",
            self.attachment, self.origin, self.clip, self.position,
        )
    }
}

impl IntoCss for Vec<Background> {
    fn into_css(self) -> String {
        self.into_iter()
            .map(IntoCss::into_css)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl CssPropertyValue for Background {
    type Kind = Background;
}
impl CssPropertyValue for Vec<Background> {
    type Kind = Background;
}
