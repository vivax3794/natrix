//! Macros for implementing a trait on specific kinds of types.

/// Call the given macro with every string type
macro_rules! strings {
    ($macro:ident) => {
        $macro!(&'static str);
        $macro!(::std::string::String);
        $macro!(::std::borrow::Cow<'static, str>);
        $macro!(::std::rc::Rc<str>);
        $macro!(::std::sync::Arc<str>);
        $macro!(::std::boxed::Box<str>);
    };
}

/// Call the given macro with every string type, but converted to a `Cow`
macro_rules! strings_cow {
    ($macro:ident) => {
        $macro!(&'static str, |this| ::std::borrow::Cow::Borrowed(this));
        $macro!(::std::string::String, |this| ::std::borrow::Cow::Owned(
            this
        ));
        $macro!(::std::borrow::Cow<'static, str>, |this| this);
        $macro!(::std::rc::Rc<str>, |this: ::std::rc::Rc<str>| {
            ::std::borrow::Cow::from(this.to_string())
        });
        $macro!(::std::sync::Arc<str>, |this: ::std::sync::Arc<str>| {
            ::std::borrow::Cow::from(this.to_string())
        });
        $macro!(::std::boxed::Box<str>, |this: ::std::boxed::Box<str>| {
            ::std::borrow::Cow::Owned(this.into_string())
        });
    };
}

/// Call the given macro with every numeric type
macro_rules! numerics {
    ($macro:ident) => {
        $macro!(u8, itoa);
        $macro!(u16, itoa);
        $macro!(u32, itoa);
        $macro!(u64, itoa);
        $macro!(u128, itoa);
        $macro!(usize, itoa);
        $macro!(i8, itoa);
        $macro!(i16, itoa);
        $macro!(i32, itoa);
        $macro!(i64, itoa);
        $macro!(i128, itoa);
        $macro!(isize, itoa);
        $macro!(f32, ryu);
        $macro!(f64, ryu);
    };
}

pub(crate) use {numerics, strings, strings_cow};
