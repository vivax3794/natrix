//! Css properties

use super::values;
use crate::css::selectors::IntoSelectorList;
use crate::css::values::IntoCss;

/// A collection of css rules
#[must_use]
pub struct RuleCollection {
    /// Raw sections of css
    pub(crate) sections: Vec<String>,
}

impl Default for RuleCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleCollection {
    /// Create a new stylesheet
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    /// Add a rule to the stylesheet
    pub fn rule(mut self, selector: impl IntoSelectorList, body: RuleBody) -> Self {
        let selector = selector.into_list().into_css();
        let body = body.into_css();

        let section = format!("{selector}{{{body}}}");
        self.sections.push(section);

        self
    }
}

impl IntoCss for RuleCollection {
    fn into_css(self) -> String {
        self.sections.join("")
    }
}

/// A impletor for a property
pub trait Property {
    /// Return the property name
    fn name(self) -> &'static str;
}

/// A marker trait that a given property supports inputs of the given type.
#[diagnostic::on_unimplemented(message = "`{Kind}` is not a valid value for this property")]
pub trait Supports<Kind>: Property {}

/// A css rule body
#[derive(Default, Clone)]
#[must_use]
pub struct RuleBody {
    /// The properties in the rule
    pub properties: Vec<(&'static str, String)>,
}

impl RuleBody {
    /// Create a empty rule body
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add on a property
    ///
    /// All defined properties have helper methods on this struct.
    #[inline]
    pub fn set<V, P>(mut self, property: P, value: V) -> Self
    where
        P: Property,
        P: Supports<V>,
        V: values::IntoCss,
    {
        self.properties.push((property.name(), value.into_css()));
        self
    }

    /// Add a raw property
    #[inline]
    pub fn raw(mut self, property: &'static str, value: impl Into<String>) -> Self {
        self.properties.push((property, value.into()));
        self
    }
}

impl IntoCss for RuleBody {
    fn into_css(self) -> String {
        let mut result = String::new();
        for (property, value) in self.properties {
            result.push_str(property);
            result.push(':');
            result.push_str(&value);
            result.push(';');
        }

        result
    }
}

/// Define a property with a specific supported value
macro_rules! property {
    ($name:ident => $target:literal) => {
        #[cfg(test)]
        static_assertions::assert_impl_all!($name: Supports<values::WideKeyword>);

        pastey::paste! {
            #[doc = "`" $target "` property."]
            #[doc = ""]
            #[doc = "<https://developer.mozilla.org/docs/Web/CSS/" $target ">"]
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct $name;

            impl Property for $name {
                #[inline]
                fn name(self) -> &'static str {
                    $target
                }
            }

            impl RuleBody {
                #[doc = "set the `" $target "` property."]
                #[doc = ""]
                #[doc = "<https://developer.mozilla.org/docs/Web/CSS/" $target ">"]
                #[inline]
                pub fn [< $name:snake >]<V>(self, value: V) -> Self
                    where $name: Supports<V>, V: values::IntoCss
                {
                    self.set($name, value)
                }
            }
        }
    };
}

/// Generate tests for property support
macro_rules! test_property {
    ($prop:ident, $value:ty, $name:ident) => {
        #[cfg(all(test, not(target_arch = "wasm32")))]
        pastey::paste! {
            proptest::proptest! {
                #[test]
                fn [< test_ $prop:snake _ $name >](value: $value) {
                    let result = RuleCollection::new()
                        .rule(crate::dom::html_elements::TagDiv, RuleBody::new().set($prop, value))
                        .into_css();
                    crate::css::assert_valid_css(&result);
                }
            }
        }
    };
}

/// Define a property support with automatic test generation
macro_rules! support {
    ($prop:ident, $value:ty, $test_name:ident) => {
        impl Supports<$value> for $prop {}
        test_property!($prop, $value, $test_name);
    };
}

impl<P: Property> Supports<values::WideKeyword> for P {}

property!(AlignContent => "align-content");
support!(AlignContent, values::Normal, normal);
support!(AlignContent, values::Stretch, stretch);
support!(AlignContent, values::ContentPosition, content);
support!(AlignContent, values::BaselinePosition, baseline);
support!(AlignContent, values::ContentDistribution, distribution);
support!(
    AlignContent,
    values::OverflowPosition<values::ContentPosition>,
    overflow
);

property!(AlignSelf => "align-self");
support!(AlignSelf, values::Auto, auto);
support!(AlignSelf, values::Normal, normal);
support!(AlignSelf, values::Stretch, stretch);
support!(AlignSelf, values::BaselinePosition, baseline);
support!(AlignSelf, values::SelfPosition, self);
support!(
    AlignSelf,
    values::OverflowPosition<values::SelfPosition>,
    overflow
);

property!(AlignItems => "align-items");
support!(AlignItems, values::Normal, normal);
support!(AlignItems, values::Stretch, stretch);
support!(AlignItems, values::SelfPosition, self);
support!(AlignItems, values::BaselinePosition, baseline);
support!(
    AlignItems,
    values::OverflowPosition<values::SelfPosition>,
    overflow
);

property!(All => "all");
test_property!(All, values::WideKeyword, wide);

property!(Animation => "animation");
support!(Animation, values::Animation, single);
support!(Animation, Vec<values::Animation>, list);

property!(Appearance => "appearance");
support!(Appearance, values::Auto, auto);
support!(Appearance, values::Appearance, special);

property!(AspectRatio => "aspect-ratio");
support!(AspectRatio, f32, f32);
support!(AspectRatio, f64, f64);
support!(AspectRatio, values::Auto, auto);
support!(AspectRatio, (values::Auto, f32), auto_f32);
support!(AspectRatio, (values::Auto, f64), auto_f64);
support!(AspectRatio, (f32, values::Auto), f32_auto);
support!(AspectRatio, (f64, values::Auto), f64_auto);
