//! Implement the various css units

use crate::css::values::ToCssValue;

/// Create a instance of a css unit, verifying at compile time the correct ranges.
#[macro_export]
macro_rules! unit {
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
pub struct Percentage(pub f32);

impl ToCssValue for Percentage {
    fn to_css(self) -> String {
        format!("{}%", self.0)
    }
}

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
