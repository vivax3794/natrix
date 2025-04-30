//! Various wrapper types to auto format css values.

use crate::component::Component;
use crate::html_elements::ToCssValue;
use crate::signal::RenderingState;
use crate::state::State;
use crate::type_macros;
use crate::utils::debug_expect;

/// A numeric css value
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Numeric<T>(pub T, pub &'static str);

/// Impl `ToCss` for all valid `Numeric`
macro_rules! generate_numeric_impl {
    ($T:ident, $fmt:ident) => {
        impl<C: Component> ToCssValue<C> for Numeric<$T> {
            fn apply_css(
                self: Box<Self>,
                name: &'static str,
                node: &web_sys::HtmlElement,
                _ctx: &mut State<C>,
                _render_state: &mut RenderingState,
            ) {
                let style = node.style();
                let res = style.set_property(name, &format!("{}{}", self.0, self.1));
                debug_expect!(res, "Failed to set css variable");
            }
        }
    };
}

type_macros::numerics!(generate_numeric_impl);

/// Generate helper functions for css units
macro_rules! css_units {
    ($($unit:ident),*) => {
        impl<T> Numeric<T> {
            $(
                #[doc = concat!("`", stringify!($unit), "`")]
                #[allow(clippy::allow_attributes, reason="This only sometimes applies")]
                #[allow(non_snake_case, reason="this is the actual unit names")]
                pub fn $unit(value: T) -> Self {
                    Numeric(value, stringify!($unit))
                }
            )*
        }
    };
}

css_units!(
    cap, ch, em, ex, ic, lh, rcap, rch, rem, rex, ric, rlh, dvh, dvw, lvh, lvw, svh, svw, vb, vh,
    vi, vmax, vmin, vw, cqb, cqh, cqi, cqmax, cqmin, cqw, cm, mm, pc, pt, px, Q, deg, grad, rad,
    turn, ms, s, Hz, kHz, fr, dpcm, dpi, ddpx
);

impl<T> Numeric<T> {
    /// `in`
    pub fn inch(value: T) -> Self {
        Numeric(value, "in")
    }

    /// `%`
    pub fn percentage(value: T) -> Self {
        Numeric(value, "%")
    }
}

/// A css color
#[derive(Clone, Copy)]
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
    #[must_use]
    pub fn with_alpha(self, alpha: f32) -> Self {
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
    #[must_use]
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::Rgb {
            red,
            green,
            blue,
            alpha: 1.0,
        }
    }

    /// Hsl with opaque alpha
    #[must_use]
    pub fn hsl(hue: u16, saturation: u8, lightness: u8) -> Self {
        Self::Hsl {
            hue,
            saturation,
            lightness,
            alpha: 1.0,
        }
    }

    /// Oklch with opaque alpha
    #[must_use]
    pub fn oklch(lightness: f32, chroma: f32, hue: f32) -> Self {
        Self::Oklch {
            lightness,
            chroma,
            hue,
            alpha: 1.0,
        }
    }

    /// Render this to a css value
    fn render(self) -> String {
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

impl<C: Component> ToCssValue<C> for Color {
    fn apply_css(
        self: Box<Self>,
        name: &'static str,
        node: &web_sys::HtmlElement,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) {
        let style = node.style();
        let res = style.set_property(name, &self.render());
        debug_expect!(res, "Failed to set css variable");
    }
}
