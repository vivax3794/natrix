//! Css properties

use super::values;

// TODO: Implement css properties

/// A impletor for a property
pub trait Property {
    /// Return the property name
    fn name(self) -> &'static str;
}

/// A marker trait that a given property supports inputs of the given type.
#[diagnostic::on_unimplemented(message = "`{Kind}` is not a valid value for this property")]
pub trait Supports<Kind>: Property {}

/// A css rule body
#[derive(Default)]
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

    /// Format this to css
    pub(super) fn into_css(self) -> String {
        let mut result = String::new();
        for (property, value) in self.properties {
            result.push_str(property);
            result.push(':');
            result.push_str(&value);
            result.push(';');
        }

        result
    }

    /// Add on a property
    ///
    /// All defined properties have helper methods on this struct.
    #[inline]
    pub fn set<Kind, P>(
        mut self,
        property: P,
        value: impl values::ToCssValue<ValueKind = Kind>,
    ) -> Self
    where
        P: Property,
        P: Supports<Kind>,
    {
        self.properties.push((property.name(), value.to_css()));
        self
    }

    /// Add a raw property
    #[inline]
    pub fn raw(mut self, property: &'static str, value: impl Into<String>) -> Self {
        self.properties.push((property, value.into()));
        self
    }
}

/// Define a property with a specific supported value
// TEST: Generate validity tests
macro_rules! property {
    ($name:ident => $target:literal) => {
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
                pub fn [< $name:snake >]<Kind>(self, value: impl values::ToCssValue<ValueKind = Kind>) -> Self
                    where $name: Supports<Kind>
                {
                    self.set($name, value)
                }
            }
        }
    };
}

impl<P: Property> Supports<values::WideKeyword> for P {}

property!(AlignContent => "align-content");
property!(AlignItems => "align-items");
property!(AlignSelf => "align-self");

impl Supports<values::Align> for AlignContent {}
impl Supports<values::Align> for AlignItems {}
impl Supports<values::Align> for AlignSelf {}

property!(All => "all");
// NOTE: `all` only accepts `WideKeyword`, hence we do not implement any `Supports` specifically
// here.
