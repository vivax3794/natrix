//! Types for the various css values

// TODO: Implement all css values
// TODO: Css variables
// TODO: Numeric calculations
// TODO: More "color" types such as gradients.

mod animations;
mod colors;

pub use colors::Color;

/// Convert a value to a css value string
pub trait ToCssValue {
    /// Convert a value to a css value string
    fn to_css(self) -> String;
}

/// Define a `ToCssValue` enum
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
            #[derive(Clone, PartialEq, Eq, Hash, Debug)]
            #[cfg_attr(all(test, not(target_arch="wasm32")), derive(proptest_derive::Arbitrary))]
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

/// Define a zero-sized css value struct
macro_rules! zero_sized_value {
    (struct $name:ident => $value:literal) => {
        pastey::paste! {
            #[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
            #[cfg_attr(
                all(test, not(target_arch = "wasm32")),
                derive(proptest_derive::Arbitrary)
            )]
            #[doc = "`" $value "`"]
            pub struct $name;

            impl ToCssValue for $name {
                #[inline]
                fn to_css(self) -> String {
                    $value.into()
                }
            }
        }
    };
}

define_enum! {
    #[derive(Copy)]
    enum WideKeyword,
    "*",
    "https://developer.mozilla.org/docs/Web/CSS/CSS_Values_and_Units/CSS_data_types#css-wide_keywords",
    {
        Initial => "initial",
        Inherit => "inherit",
        Revert => "revert",
        RevertLayer => "revert-layer",
        Unset => "unset",
    }
}

define_enum! {
    #[derive(Copy)]
    enum ContentPosition,
    "align-*",
    "https://www.w3.org/TR/css-align-3/#typedef-content-position",
    {
        Center => "center",
        Start => "start",
        End => "end",
        FlexStart => "flex-start",
        FlexEnd => "flex-end",
    }
}

define_enum! {
    #[derive(Copy)]
    enum BaselinePosition,
    "align-*",
    "https://www.w3.org/TR/css-align-3/#typedef-baseline-position",
    {
        Baseline => "baseline",
        First => "first baseline",
        // SPEC: Last-baseline not supported on `align-content`
        Last => "last baseline",
    }
}

define_enum! {
    #[derive(Copy)]
    enum ContentDistribution,
    "align-*",
    "https://www.w3.org/TR/css-align-3/#typedef-content-distribution",
    {
        SpaceBetween => "space-between",
        SpaceAround => "space-around",
        SpaceEvenly => "space-evenly",
    }
}

define_enum! {
    #[derive(Copy)]
    enum SelfPosition,
    "align-*",
    "https://www.w3.org/TR/css-align-3/#typedef-self-position",
    {
        Center => "center",
        Start => "Start",
        End => "End",
        SelfStart => "self-start",
        SelfEnd => "self-end",
        FlexStart => "flex-start",
        FlexEnd => "flex-end"
    }
}

zero_sized_value!(struct Normal => "normal");
zero_sized_value!(struct Auto => "auto");
zero_sized_value!(struct Stretch => "stretch");

/// <https://developer.mozilla.org/en-US/docs/Web/CSS/align-content#safe>
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(
    all(test, not(target_arch = "wasm32")),
    derive(proptest_derive::Arbitrary)
)]
pub enum OverflowPosition<T> {
    /// `safe ...`
    Safe(T),
    /// `unsafe ...`
    Unsafe(T),
}

impl<T: ToCssValue> ToCssValue for OverflowPosition<T> {
    fn to_css(self) -> String {
        let (prefix, value) = match self {
            Self::Safe(value) => ("safe", value),
            Self::Unsafe(value) => ("unsafe", value),
        };

        format!("{prefix} {}", value.to_css())
    }
}
