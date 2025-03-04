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

/// Call the given macro with every int type
macro_rules! ints {
    ($macro:ident) => {
        $macro!(u8);
        $macro!(u16);
        $macro!(u32);
        $macro!(u64);
        $macro!(u128);
        $macro!(usize);
        $macro!(i8);
        $macro!(i16);
        $macro!(i32);
        $macro!(i64);
        $macro!(i128);
        $macro!(isize);
    };
}

pub(crate) use {ints, strings};
