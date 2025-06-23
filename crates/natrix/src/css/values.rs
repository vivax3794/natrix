//! Types for the various css values

// TODO: Implement all css values
// TODO: Css variables
// TODO: Numeric calculations
// TODO: More "color" types such as gradients.

mod colors;
pub use colors::Color;

/// Convert a value to a css value string
pub trait ToCssValue {
    /// The kind of value this is
    type ValueKind;

    /// Convert a value to a css value string
    fn to_css(self) -> String;
}

/// Define a `ToCssValue` enum
// TEST: Auto generate tests for validity.
macro_rules! define_enum {
    (
        $(#[$enum_meta:meta])*
        enum $name:ident,
        $value_name:literal,
        $mdn_url:literal,
        {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $string_value:literal
            ),*
            $(, | $other:ident)?
            $(,)? // Optional trailing comma
        }
    ) => {
        pastey::paste! {
            #[doc = "Value for the `" $value_name "` css property"]
            #[doc = ""]
            #[doc = "<" $mdn_url ">"]
            $(#[$enum_meta])*
            #[derive(Clone, PartialEq, Eq, Hash)]
            pub enum $name {
                $(
                    $(#[$variant_meta])*
                    #[doc = "`" $string_value "`"]
                    $variant,
                )*
                $(
                    #[doc = "Custom value"]
                    $other(String),
                )?
            }

            impl ToCssValue for $name {
                type ValueKind = $name;

                #[inline]
                fn to_css(self) -> String {
                    match self {
                        $(
                            Self::$variant => $string_value.into(),
                        )*
                        $(
                            Self::$other(value) => value.into()
                        )?
                    }
                }
            }
        }
    }
}

define_enum! {
    #[derive(Copy)]
    enum WideKeyword,
    "*",
    "https://developer.mozilla.org/docs/Web/CSS/CSS_Values_and_Units/CSS_data_types#css-wide_keywords",
    {
        Initial => "inital",
        Inherit => "inherit",
        Revert => "revert",
        RevertLayer => "revert-layer",
        Unset => "unset",
    }
}

define_enum! {
    #[derive(Copy, Default)]
    enum Align,
    "align-*",
    "https://developer.mozilla.org/en-US/docs/Web/CSS/align-content#values",
    {
        #[default]
        Normal => "normal",
        Start => "start",
        Center => "center",
        End => "end",
        FlexStart => "flex-start",
        FlexEnd => "flex-end",
        Baseline => "baseline",
        FirstBaseline => "first baseline",
        // SPEC: These are noops on `align-content` in block layouts.
        SpaceBetween => "space-between",
        SpaceAround => "space-around",
        SpaceEvenly => "space-evenly",
        Stretch => "stretch"
    }
}

/// <https://developer.mozilla.org/en-US/docs/Web/CSS/align-content#safe>
pub struct Safe(pub Align);

/// <https://developer.mozilla.org/en-US/docs/Web/CSS/align-content#unsafe>
// MAYBE: Should we name this something else? considering `unsafe` has connotation in rust.
pub struct Unsafe(pub Align);

impl ToCssValue for Safe {
    type ValueKind = Align;

    fn to_css(self) -> String {
        format!("safe {}", self.0.to_css())
    }
}
impl ToCssValue for Unsafe {
    type ValueKind = Align;

    fn to_css(self) -> String {
        format!("unsafe {}", self.0.to_css())
    }
}
