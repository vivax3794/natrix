//! Convert various values to html attributes

use std::borrow::Cow;

use wasm_bindgen::intern;

use super::html_elements::DeferredFunc;
use crate::error_handling::log_or_panic;
use crate::reactivity::component::Component;
use crate::reactivity::render_callbacks::{ReactiveAttribute, SimpleReactive};
use crate::reactivity::state::RenderCtx;
use crate::type_macros;

/// The result of apply attribute
pub(crate) enum AttributeResult<C: Component> {
    /// The attribute was set
    SetIt(Option<Cow<'static, str>>),
    /// The attribute was dynamic
    IsDynamic(DeferredFunc<C>),
}

/// A trait for using a arbitrary type as a attribute value.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid attribute value.",
    note = "Try converting the value to a string"
)]
pub trait ToAttribute<C: Component>: 'static {
    /// The kind of attribute output this is
    type AttributeKind;

    /// Modify the given node to have the attribute set
    ///
    /// We use this apply system instead of returning the value as some types will also need to
    /// conditionally remove the attribute
    fn calc_attribute(self, name: &'static str, node: &web_sys::Element) -> AttributeResult<C>;
}

/// A attribute that is a integer
pub struct Integer;

/// A attribute that is a float
pub struct Float;

/// generate a `ToAttribute` implementation for a string type
macro_rules! attribute_string {
    ($t:ty, $cow:expr) => {
        impl<C: Component> ToAttribute<C> for $t {
            type AttributeKind = String;

            fn calc_attribute(
                self,
                _name: &'static str,
                _node: &web_sys::Element,
            ) -> AttributeResult<C> {
                AttributeResult::SetIt(Some(($cow)(self)))
            }
        }
    };
}

type_macros::strings_cow!(attribute_string);

impl<C: Component> ToAttribute<C> for char {
    type AttributeKind = char;

    fn calc_attribute(self, _name: &'static str, _node: &web_sys::Element) -> AttributeResult<C> {
        AttributeResult::SetIt(Some(Cow::from(self.to_string())))
    }
}

/// generate `ToAttribute` for a int using itoa
macro_rules! attribute_int {
    ($t:ident, $fmt:ident, $name:ident) => {
        impl<C: Component> ToAttribute<C> for $t {
            type AttributeKind = $name;

            fn calc_attribute(
                self,
                _name: &'static str,
                _node: &web_sys::Element,
            ) -> AttributeResult<C> {
                let mut buffer = $fmt::Buffer::new();
                let result = buffer.format(self);

                AttributeResult::SetIt(Some(Cow::from(result.to_string())))
            }
        }
    };
}

type_macros::numerics!(attribute_int);

impl<C: Component> ToAttribute<C> for bool {
    type AttributeKind = bool;

    fn calc_attribute(self, _name: &'static str, _node: &web_sys::Element) -> AttributeResult<C> {
        AttributeResult::SetIt(self.then(|| Cow::from("")))
    }
}

impl<C: Component, T: ToAttribute<C>> ToAttribute<C> for Option<T> {
    type AttributeKind = T::AttributeKind;

    fn calc_attribute(self, name: &'static str, node: &web_sys::Element) -> AttributeResult<C> {
        if let Some(inner) = self {
            inner.calc_attribute(name, node)
        } else {
            AttributeResult::SetIt(None)
        }
    }
}

impl<C: Component, T: ToAttribute<C, AttributeKind = K>, E: ToAttribute<C, AttributeKind = K>, K>
    ToAttribute<C> for Result<T, E>
{
    type AttributeKind = K;

    fn calc_attribute(self, name: &'static str, node: &web_sys::Element) -> AttributeResult<C> {
        match self {
            Ok(inner) => inner.calc_attribute(name, node),
            Err(inner) => inner.calc_attribute(name, node),
        }
    }
}

impl<F, C, R> ToAttribute<C> for F
where
    F: Fn(&mut RenderCtx<C>) -> R + 'static,
    R: ToAttribute<C>,
    C: Component,
{
    type AttributeKind = R::AttributeKind;

    fn calc_attribute(self, name: &'static str, node: &web_sys::Element) -> AttributeResult<C> {
        let node = node.clone();

        AttributeResult::IsDynamic(Box::new(move |ctx, render_state| {
            let hook = SimpleReactive::init_new(
                Box::new(move |ctx| ReactiveAttribute {
                    name,
                    data: self(ctx),
                }),
                node.clone(),
                ctx,
            );
            render_state.hooks.push(hook);
        }))
    }
}

/// Defines an enum that represents the value of an enumerated HTML attribute.
macro_rules! define_attribute_enum {
    (
        $(#[$enum_meta:meta])*
        enum $name:ident,
        $attribute_name:literal,
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
            #[doc = "Value for the `" $attribute_name "` attribute"]
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
                    $other(Cow<'static, str>),
                )?
            }

            impl<C: Component> ToAttribute<C> for $name {
                type AttributeKind = $name;

                #[inline]
                fn calc_attribute(self, _name: &'static str, _node: &web_sys::Element) -> AttributeResult<C> {
                    AttributeResult::SetIt(Some(match self {
                        $(
                            Self::$variant => intern($string_value).into(),
                        )*
                        $(
                            Self::$other(value) => value
                        )?
                    }))
                }
            }
        }
    };
}

/// Impl `ToAttribute` for a vec for another attribute using a space separated list
#[macro_export]
macro_rules! impl_to_attribute_for_vec {
    ($T:ty) => {
        impl<C: Component> ToAttribute<C> for Vec<$T> {
            type AttributeKind = $T;

            fn calc_attribute(
                self,
                name: &'static str,
                node: &web_sys::Element,
            ) -> AttributeResult<C> {
                let result = self
                    .into_iter()
                    .map(|item| {
                        if let AttributeResult::SetIt(Some(value)) =
                            ToAttribute::<C>::calc_attribute(item, name, node)
                        {
                            value
                        } else {
                            log_or_panic!(concat!(
                                "`",
                                stringify!($T),
                                "` produced dynamic value or None"
                            ));
                            Cow::from("")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                AttributeResult::SetIt(Some(Cow::from(result)))
            }
        }
    };
}

/// Define a boolean attribute.
/// This is a attribute thats effectively a bool.
/// but the html spec in its infinite wisdom uses a unique set of enumerated values for
macro_rules! define_bool_attribute {
    ($struct_name:ident, $true_str:tt, $false_str:tt) => {
        /// A boolean-like attribute.
        #[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct $struct_name(pub bool);

        impl<C: Component> ToAttribute<C> for $struct_name {
            type AttributeKind = $struct_name;

            fn calc_attribute(
                self,
                _name: &'static str,
                _node: &web_sys::Element,
            ) -> AttributeResult<C> {
                AttributeResult::SetIt(Some(Cow::from(if self.0 {
                    intern($true_str)
                } else {
                    intern($false_str)
                })))
            }
        }
    };
}

// has html ever heard of DRY?!
define_bool_attribute!(TrueFalse, "true", "false");
define_bool_attribute!(YesNo, "yes", "no");
define_bool_attribute!(OnOff, "on", "off");

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum ContentEditable,
    "contenteditable",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Global_attributes/contenteditable#value",
    {
        True => "true",
        #[default]
        False => "false",
        PlaintextOnly => "plaintext-only",
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum Direction,
    "dir",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Global_attributes/dir",
    {
        LeftToRight => "ltr",
        RightToLeft => "rtl",
        #[default]
        Auto => "auto",
    }
}

define_attribute_enum! {
    #[derive(Copy)]
    enum EnterkeyHint,
    "enterkeyhint",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Global_attributes/enterkeyhint#values",
    {
        Enter => "enter",
        Done => "done",
        Go => "go",
        Next => "next",
        Previous => "previous",
        Search => "search",
        Send => "send",
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum InputMode,
    "inputmode",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Global_attributes/inputmode#values",
    {
        None => "none",
        #[default]
        Text => "text",
        Decimal => "decimal",
        Numeric => "numeric",
        Telephone => "tel",
        Search => "search",
        Email => "email",
        Url => "url",
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum PopOver,
    "popover",
    "https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/popover#value",
    {
        #[default]
        Auto => "auto",
        Manual => "manual"
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum ReferrerPolicy,
    "referrerpolicy",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/a#referrerpolicy",
    {
        NoReferrer => "no-referrer",
        NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
        Origin => "origin",
        OriginWhenCrossOrigin => "origin-when-cross-origin",
        SameOrigin => "same-origin",
        StrictOrigin => "strict-origin",
        #[default]
        StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
        UnsafeUrl => "unsafe-url"
    }
}

define_attribute_enum! {
    enum Rel,
    "rel",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Attributes/rel",
    {
        NoOpener => "noopener",
        NoReferrer => "noreferrer",
        | Other
    }
}

impl_to_attribute_for_vec!(Rel);

define_attribute_enum! {
    #[derive(Default)]
    enum Target,
    "target",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/a#target",
    {
        #[default]
        This => "_self",
        NewTab => "_blank",
        Parent => "_parent",
        Top => "_top",
        UnfencedTop => "_unfencedTop",
        | Other,
    }
}

define_attribute_enum! {
    #[derive(Copy)]
    enum Shape,
    "shape",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/area#shape",
    {
        Rectangle => "rect",
        Circle => "circle",
        Polygon => "poly",
        Remaining => "default"
    }
}

define_attribute_enum! {
    #[derive(Copy)]
    enum ControlsList,
    "controlslist",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/audio#controlslist",
    {
        NoDownload => "nodownload",
        NoFullscreen => "nofullscreen",
        NoRemotePlayback => "noremoteplayback"
    }
}

impl_to_attribute_for_vec!(ControlsList);

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum CrossOrigin,
    "crossorigin",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Attributes/crossorigin",
    {
        #[default]
        Anonymous => "anonymous",
        UseCredentials => "use-credentials"
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum AudioPreload,
    "preload",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/audio#preload",
    {
        None => "none",
        #[default]
        Metadata => "metadata",
        Auto => "auto"
    }
}

define_attribute_enum! {
    #[derive(Copy)]
    enum Command,
    "command",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/button#command",
    {
        ShowModal => "show-modal",
        Close => "close",
        RequestClose => "request-close",
        ShowPopover => "show-popover",
        HidePopover => "hide-popover",
        TogglePopover => "toggle-popover",
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum EncodingType,
    "enctype",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/form#enctype",
    {
        #[default]
        FormUrlEncoded => "application/x-www-form-urlencoded",
        MultipartFormData => "multipart/form-data",
        PlainText => "text/plain"
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum FormMethod,
    "method",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/form#method",
    {
        Post => "post",
        #[default]
        Get => "get",
        Dialog => "dialog",
        Submit => "submit"
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum PopoverAction,
    "popovertargetaction",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/button#popovertargetaction",
    {
        Hide => "hide",
        Show => "show",
        #[default]
        Toggle => "toggle"
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum ButtonType,
    "type",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/button#type",
    {
        Submit => "submit",
        Reset => "reset",
        #[default]
        Button => "button"
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum AutoCapitalize,
    "autocapitalize",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Global_attributes/autocapitalize",
    {
        #[default]
        Off => "off",
        Sentences => "sentences",
        Words => "words",
        Characters => "characters"
    }
}

define_attribute_enum! {
    #[derive(Default, Copy)]
    enum IframeLoading,
    "loading",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/iframe#loading",
    {
        #[default]
        Eager => "eager",
        Lazy => "lazy",
    }
}

define_attribute_enum! {
    #[derive(Copy)]
    enum SandboxAllow,
    "sandbox",
    "https://developer.mozilla.org/docs/Web/HTML/Reference/Elements/iframe#sandbox",
    {
        Downloads => "allow-downloads",
        Forms => "allow-forms",
        Modals => "allow-modals",
        OrientationLock => "allow-orientation-lock",
        PointerLock => "allow-pointer-lock",
        Popups => "allow-popups",
        PopupsToEscapeSandbox => "allow-popups-to-escape-sandbox",
        Presentation => "allow-presentation",
        SameOrigin => "allow-same-origin",
        Scripts => "allow-scripts",
        TopNavigation => "allow-top-navigation",
        TopNavigationByUserActivation => "allow-top-navigation-by-user-activation",
        TopNavigationToCustomProtocols => "allow-top-navigation-to-custom-protocols",
    }
}

impl_to_attribute_for_vec!(SandboxAllow);
