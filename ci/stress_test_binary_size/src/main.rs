/*
use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

use natrix::prelude::*;
use pastey::paste;

macro_rules! large_type {
    ($name:ident,$($field:ident.$type:ty), *) => {
        #[derive(State, Default)]
        struct $name{
            $(
                $field: Signal<$type>
            ),*
        }

        paste! {
            fn [<render_ $name:lower>]() -> impl Element<$name> {
                e::div()
                    $(
                        .child(|ctx: &mut RenderCtx<$name>| ctx.$field.clone())
                        .attr(stringify!($field), |ctx: &mut RenderCtx<$name>| ctx.$field.clone())
                    )*
            }
        }
    };
}

macro_rules! define_root_test {
    ($($child:ident),*) => {
        #[derive(State)]
        struct Root;

        paste! {
            fn render_root() -> impl Element<Root> {
                e::div()
                    $(
                        .child([<render_ $child:lower>]())
                    )*
            }
        }
    };
}

large_type!(A1, a.u8, b.u8, c.u8, d.u8, e.u8);
large_type!(A2, a.u8, b.u8, c.u8, d.u8, e.u8);
large_type!(A3, a.u8, b.u8, c.u8, d.u8, e.u8);
large_type!(A4, a.u8, b.u8, c.u8, d.u8, e.u8);
large_type!(A5, a.u8, b.u8, c.u8, d.u8, e.u8);

large_type!(B1, a.u8, b.u16, c.u32, d.u64, e.u128);
large_type!(B2, a.u8, b.u16, c.u32, d.u64, e.u128);
large_type!(B3, a.u8, b.u16, c.u32, d.u64, e.u128);
large_type!(B4, a.u8, b.u16, c.u32, d.u64, e.u128);
large_type!(B5, a.u8, b.u16, c.u32, d.u64, e.u128);

large_type!(C1, a.Option<u8>, b.Option<u16>, c.Option<u32>, d.Option<u64>, e.Option<u128>);
large_type!(C2, a.Option<u8>, b.Option<u16>, c.Option<u32>, d.Option<u64>, e.Option<u128>);
large_type!(C3, a.Option<u8>, b.Option<u16>, c.Option<u32>, d.Option<u64>, e.Option<u128>);
large_type!(C4, a.Option<u8>, b.Option<u16>, c.Option<u32>, d.Option<u64>, e.Option<u128>);
large_type!(C5, a.Option<u8>, b.Option<u16>, c.Option<u32>, d.Option<u64>, e.Option<u128>);

large_type!(D1, a.String, b.Cow<'static, str>, c.Rc<str>, d.Arc<str>, e.Box<str>);
large_type!(D2, a.String, b.Cow<'static, str>, c.Rc<str>, d.Arc<str>, e.Box<str>);
large_type!(D3, a.String, b.Cow<'static, str>, c.Rc<str>, d.Arc<str>, e.Box<str>);
large_type!(D4, a.String, b.Cow<'static, str>, c.Rc<str>, d.Arc<str>, e.Box<str>);
large_type!(D5, a.String, b.Cow<'static, str>, c.Rc<str>, d.Arc<str>, e.Box<str>);

large_type!(E1, a.u8, b.String, c.i8, d.Cow<'static, str>, e.Option<String>);
large_type!(E2, a.i32, b.Rc<str>, c.u8, d.Arc<str>, e.Option<Option<Box<str>>>);
large_type!(E3, a.i128, b.u8, c.String, d.Cow<'static, str>, e.Option<u8>);
large_type!(E4, a.u8, b.Rc<str>, c.i8, d.Arc<str>, e.Option<String>);
large_type!(E5, a.i32, b.u8, c.String, d.Cow<'static, str>, e.Option<u8>);

define_root_test!(
    A1, A2, A3, A4, A5, B1, B2, B3, B4, B5, C1, C2, C3, C4, C5, D1, D2, D3, D4, D5, E1, E2, E3, E4,
    E5
);
*/

fn main() {
    // Stress test disabled until sub-states are supported
}
