//! Implementation for css colors

use super::ToCssValue;

/// A css color
///
/// # Important
/// All color methods and constructor use `debug_assert` to verify input ranges. (as css itself
/// handles out of range colors fine in prod).
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub enum Color {
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/color_value/rgb>
    Rgb {
        /// 0-255
        red: u8,
        /// 0-255
        green: u8,
        /// 0-255
        blue: u8,
        /// 0-1
        alpha: f32,
    },
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/color_value/hsl>
    Hsl {
        /// 0-360
        hue: u16,
        /// 0-100
        saturation: u8,
        /// 0-100
        lightness: u8,
        /// 0-1
        alpha: f32,
    },
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/color_value/oklch>
    Oklch {
        /// 0 - 1
        lightness: f32,
        /// 0 - 1
        chroma: f32,
        /// 0 - 1
        hue: f32,
        /// 0 - 1
        alpha: f32,
    },
}

impl Color {
    /// Set the alpha and return a new color
    pub const fn with_alpha(self, alpha: f32) -> Self {
        debug_assert!(alpha <= 1.0 && alpha >= 0.0, "Alpha should between 0-1");

        match self {
            Self::Rgb {
                red,
                green,
                blue,
                alpha: _,
            } => Self::Rgb {
                red,
                green,
                blue,
                alpha,
            },
            Self::Hsl {
                hue,
                saturation,
                lightness,
                alpha: _,
            } => Self::Hsl {
                hue,
                saturation,
                lightness,
                alpha,
            },
            Self::Oklch {
                lightness,
                chroma,
                hue,
                alpha: _,
            } => Self::Oklch {
                lightness,
                chroma,
                hue,
                alpha,
            },
        }
    }

    /// Rgb with opaque alpha
    #[inline]
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, 1.0)
    }

    /// Rgb with alpha
    #[inline]
    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: f32) -> Self {
        debug_assert!(alpha <= 1.0 && alpha >= 0.0, "Alpha should be in range 0-1");

        Color::Rgb {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Hsl with opaque alpha
    #[inline]
    pub const fn hsl(hue: u16, saturation: u8, lightness: u8) -> Self {
        Self::hsla(hue, saturation, lightness, 1.0)
    }

    /// Hsl with a given alpha
    #[inline]
    pub const fn hsla(hue: u16, saturation: u8, lightness: u8, alpha: f32) -> Self {
        debug_assert!(hue <= 360, "Hue should be in range 0-360");
        debug_assert!(saturation <= 100, "saturation should be in range 0-100");
        debug_assert!(lightness <= 100, "lightness should be in range 0-100");
        debug_assert!(alpha <= 1.0 && alpha >= 0.0, "Alpha should be in range 0-1");

        Self::Hsl {
            hue,
            saturation,
            lightness,
            alpha,
        }
    }

    /// Oklch with opaque alpha
    #[inline]
    pub const fn oklch(lightness: f32, chroma: f32, hue: f32) -> Self {
        Self::Oklch {
            lightness,
            chroma,
            hue,
            alpha: 1.0,
        }
    }

    /// Oklch with a given alpha
    #[inline]
    pub const fn oklch_a(lightness: f32, chroma: f32, hue: f32, alpha: f32) -> Self {
        debug_assert!(
            lightness <= 1.0 && lightness >= 0.0,
            "Lightness should be in range 0-1"
        );
        debug_assert!(
            chroma <= 1.0 && chroma >= 0.0,
            "chroma should be in range 0-1"
        );
        debug_assert!(hue <= 1.0 && hue >= 0.0, "hue should be in range 0-1");
        debug_assert!(alpha <= 1.0 && alpha >= 0.0, "Alpha should be in range 0-1");

        Self::Oklch {
            lightness,
            chroma,
            hue,
            alpha,
        }
    }
}

impl ToCssValue for Color {
    fn to_css(self) -> String {
        match self {
            Self::Rgb {
                red,
                green,
                blue,
                alpha,
            } => {
                format!("rgb({red} {green} {blue}/{alpha})",)
            }
            Self::Hsl {
                hue,
                saturation,
                lightness,
                alpha,
            } => {
                format!("hsl({hue} {saturation} {lightness}/{alpha})")
            }
            Self::Oklch {
                lightness,
                chroma,
                hue,
                alpha,
            } => {
                format!("oklch({lightness} {chroma} {hue}/{alpha})")
            }
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) mod tests {
    use insta::assert_snapshot;
    use proptest::prelude::*;

    use super::*;
    use crate::css::assert_valid_css;

    impl Arbitrary for Color {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                // Strategy for Rgb variant
                (0u8..=255, 0u8..=255, 0u8..=255, 0.0f32..=1.0).prop_map(
                    |(red, green, blue, alpha)| Color::Rgb {
                        red,
                        green,
                        blue,
                        alpha,
                    }
                ),
                // Strategy for Hsl variant
                (0u16..=360, 0u8..=100, 0u8..=100, 0.0f32..=1.0).prop_map(
                    |(hue, saturation, lightness, alpha)| Color::Hsl {
                        hue,
                        saturation,
                        lightness,
                        alpha,
                    }
                ),
                // Strategy for Oklch variant
                (0.0f32..=1.0, 0.0f32..=1.0, 0.0f32..=1.0, 0.0f32..=1.0).prop_map(
                    |(lightness, chroma, hue, alpha)| Color::Oklch {
                        lightness,
                        chroma,
                        hue,
                        alpha,
                    }
                ),
            ]
            .boxed()
        }
    }

    #[test]
    fn color_with_alpha() {
        let direct = Color::rgba(100, 100, 100, 0.5);
        let chained = Color::rgb(100, 100, 100).with_alpha(0.5);

        assert_eq!(direct, chained);
    }

    #[test]
    fn snapshot_rgb_colors() {
        assert_snapshot!("rgb_red", Color::rgb(255, 0, 0).to_css());
        assert_snapshot!("rgb_green", Color::rgb(0, 255, 0).to_css());
        assert_snapshot!("rgb_blue", Color::rgb(0, 0, 255).to_css());
        assert_snapshot!("rgb_black", Color::rgb(0, 0, 0).to_css());
        assert_snapshot!("rgb_white", Color::rgb(255, 255, 255).to_css());
    }

    #[test]
    fn snapshot_rgba_colors() {
        assert_snapshot!("rgba_half_alpha", Color::rgba(255, 0, 0, 0.5).to_css());
        assert_snapshot!("rgba_zero_alpha", Color::rgba(0, 255, 0, 0.0).to_css());
        assert_snapshot!("rgba_full_alpha", Color::rgba(0, 0, 255, 1.0).to_css());
    }

    #[test]
    fn snapshot_hsl_colors() {
        assert_snapshot!("hsl_red", Color::hsl(0, 100, 50).to_css());
        assert_snapshot!("hsl_green", Color::hsl(120, 100, 50).to_css());
        assert_snapshot!("hsl_blue", Color::hsl(240, 100, 50).to_css());
        assert_snapshot!("hsl_gray", Color::hsl(0, 0, 50).to_css());
    }

    #[test]
    fn snapshot_hsla_colors() {
        assert_snapshot!("hsla_half_alpha", Color::hsla(0, 100, 50, 0.5).to_css());
        assert_snapshot!("hsla_zero_alpha", Color::hsla(120, 100, 50, 0.0).to_css());
        assert_snapshot!("hsla_full_alpha", Color::hsla(240, 100, 50, 1.0).to_css());
    }

    #[test]
    fn snapshot_oklch_colors() {
        assert_snapshot!("oklch_mid_lightness", Color::oklch(0.5, 0.1, 0.5).to_css());
        assert_snapshot!("oklch_low_lightness", Color::oklch(0.1, 0.2, 0.8).to_css());
        assert_snapshot!(
            "oklch_high_lightness",
            Color::oklch(0.9, 0.05, 0.2).to_css()
        );
    }

    #[test]
    fn snapshot_oklch_a_colors() {
        assert_snapshot!(
            "oklch_a_half_alpha",
            Color::oklch_a(0.5, 0.1, 0.5, 0.5).to_css()
        );
        assert_snapshot!(
            "oklch_a_zero_alpha",
            Color::oklch_a(0.1, 0.2, 0.8, 0.0).to_css()
        );
        assert_snapshot!(
            "oklch_a_full_alpha",
            Color::oklch_a(0.9, 0.05, 0.2, 1.0).to_css()
        );
    }

    #[test]
    fn snapshot_with_alpha_method() {
        assert_snapshot!(
            "with_alpha_rgb",
            Color::rgb(10, 20, 30).with_alpha(0.75).to_css()
        );
        assert_snapshot!(
            "with_alpha_hsl",
            Color::hsl(90, 50, 25).with_alpha(0.25).to_css()
        );
        assert_snapshot!(
            "with_alpha_oklch",
            Color::oklch(0.3, 0.1, 0.6).with_alpha(0.9).to_css()
        );
    }

    proptest! {
        #[test]
        #[cfg(not(debug_assertions))]
        fn color_hsl_doesnt_crash_in_prod(hue: u16, sat: u8, lit: u8) {
            let _ = Color::hsl(hue, sat, lit);
        }

        #[test]
        #[cfg(not(debug_assertions))]
        fn color_oklch_doesnt_crash_in_prod(lit: f32, chroma: f32, hue: f32) {
            let _ = Color::oklch(lit, chroma, hue);
        }

        #[test]
        fn render_colors(color: Color) {
            let color = color.to_css();
            let wrapping_css = format!("h1 {{background-color: {color};}}");

            assert_valid_css(&wrapping_css);
        }
    }
}
