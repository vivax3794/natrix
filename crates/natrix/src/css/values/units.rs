//! Implement the various css units
use crate::css::values::IntoCss;

/// Create a instance of a css unit, verifying at compile time the correct ranges.
#[macro_export]
macro_rules! percentage {
    ($value:literal %) => {{
        #[expect(dead_code, reason="cargo check/clippy only checks const expressions if actually assigned to a constant.")]
        const CHECK: () = const {
            debug_assert!($value >= 0.0, "percentage must be in range 0-100");
            debug_assert!($value <= 100.0, "percentage must be in range 0-100");
        };
        $crate::css::values::units::Percentage($value)
    }};
}

/// A css percentage.
/// For compile-time validating a valid percentage use `unit!` macro
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
#[cfg_attr(
    all(test, not(target_arch = "wasm32")),
    derive(proptest_derive::Arbitrary)
)]
pub struct Percentage(pub f32);

impl IntoCss for Percentage {
    fn into_css(self) -> String {
        format!("{}%", self.0)
    }
}

/// A css `<length>` value,
#[derive(Clone, Copy, PartialEq, Debug)]
#[cfg_attr(
    all(test, not(target_arch = "wasm32")),
    derive(proptest_derive::Arbitrary)
)]
pub struct Length {
    /// The value itself
    pub value: f64,
}

/*
define_enum! {
    #[derive(Copy)]
    enum LengthUnit,
    "*",
    "https://developer.mozilla.org/en-US/docs/Web/CSS/length",
    {
        CapitalHeight => "cap",
        Character => "ch",
        FontSize => "em",
        Xheight => "ex",
        IdealCharacter => "ic",
        Lineheight => "lh",
        RootCapHeight => "rcap",
        RootCharacter => "rch",
        RootFontSize => "rem",
        RootXheight => "rex",
        RootIdealCharacter => "ric",
        RootLineheight => "rlh",
        ContainerQueryWidth => "cqw",
        ContainerQueryHeight => "cqh",
        ContainerQueryInlineSize => "cqi",
        ContainerQueryBlockSize => "cqb",
        ContainerQueryMax => "cqmax",
        ContainerQueryMin => "cqmin",
        Pixel => "px",
        CentiMeter => "cm",
        Millimeter => "mm",
        QuarterMillimeter => "Q",
        Inch => "in",
        Pica => "pc",
        Point => "pt",
        ViewportHeight => "vh",
        ViewportWidth => "vw",
        ViewportMax => "vmax",
        ViewportMin => "vmin",
        ViewportBlockAxis => "vb",
        ViewportInlineAxis => "vi",
        SmallViewportHeight => "svh",
        SmallViewportWidth => "svw",
        SmallViewportMax => "svmax",
        SmallViewportMin => "svmin",
        SmallViewportBlockAxis => "svb",
        SmallViewportInlineAxis => "svi",
        LargeViewportHeight => "lvh",
        LargeViewportWidth => "lvw",
        LargeViewportMax => "lvmax",
        LargeViewportMin => "lvmin",
        LargeViewportBlockAxis => "lvb",
        LargeViewportInlineAxis => "lvi",
        DynamicViewportHeight => "dvh",
        DynamicViewportWidth => "dvw",
        DynamicViewportMax => "dvmax",
        DynamicViewportMin => "dvmin",
        DynamicViewportBlockAxis => "dvb",
        DynamicViewportInlineAxis => "dvi",
    }
}
*/

/// ```compile_fail
/// use natrix::unit;
/// let x = unit!(200.0%);
/// ```
/// ```compile_fail
/// use natrix::unit;
/// let x = unit!(-10.0%);
/// ```
#[expect(dead_code, reason = "For compile fail tests only")]
fn compile_fail() {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_cases() {
        assert_eq!(unit!(0.0%), Percentage(0.0));
        assert_eq!(unit!(100.0%), Percentage(100.0));
        assert_eq!(unit!(50.0%), Percentage(50.0));
    }
}
