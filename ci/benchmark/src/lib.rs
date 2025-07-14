#![feature(custom_inner_attributes)]
#![rust_analyzer::skip] // The macros in here make it slow
#![allow(dead_code, reason = "test")]

use natrix::class;
use natrix::prelude::*;

mod runtime;

const BUTTON: Id = natrix::id!();
const BUTTON2: Id = natrix::id!();

#[derive(State, Default)]
struct Buttons<const N: u32> {
    state: Signal<u32>,
}

fn render_buttons<const N: u32>() -> impl Element<Buttons<N>> {
    let mut res = e::div();

    for _ in 0..N {
        res = res.child(
            e::button()
                .id(BUTTON)
                .text(|ctx: RenderCtx<Buttons<N>>| *ctx.state)
                .on::<events::Click>(|mut ctx: EventCtx<Buttons<N>>, _| {
                    *ctx.state += 1;
                }),
        );
    }

    res
}

#[derive(State, Default)]
struct ToggleNode<const N: u32> {
    state: Signal<bool>,
}

fn render_toggle_node<const N: u32>() -> impl Element<ToggleNode<N>> {
    let mut res = e::div().child(e::button().id(BUTTON).on::<events::Click>(
        |mut ctx: EventCtx<ToggleNode<N>>, _| {
            *ctx.state = !*ctx.state;
        },
    ));

    for _ in 0..N {
        res = res.child(e::div().child(|ctx: RenderCtx<ToggleNode<N>>| {
            // NOTE: In a real application the reactivity would be on the text level
            // But we are testing dom swapping.
            if *ctx.state {
                e::h1().text("ON").generic()
            } else {
                e::h2().text("OFF").generic()
            }
        }));
    }

    res
}

#[derive(State, Default)]
struct ToggleText<const N: u32> {
    state: Signal<bool>,
}

fn render_toggle_text<const N: u32>() -> impl Element<ToggleText<N>> {
    let mut res = e::div().child(e::button().id(BUTTON).on::<events::Click>(
        |mut ctx: EventCtx<ToggleText<N>>, _| {
            *ctx.state = !*ctx.state;
        },
    ));

    for _ in 0..N {
        res = res.child(
            e::div().child(|ctx: RenderCtx<ToggleText<N>>| if *ctx.state { "ON" } else { "OFF" }),
        );
    }

    res
}

#[derive(State, Default)]
struct ToggleAttr<const N: u32> {
    state: Signal<bool>,
}

fn render_toggle_attr<const N: u32>() -> impl Element<ToggleAttr<N>> {
    let mut res = e::div().child(e::button().id(BUTTON).on::<events::Click>(
        |mut ctx: EventCtx<ToggleAttr<N>>, _| {
            *ctx.state = !*ctx.state;
        },
    ));

    for _ in 0..N {
        res = res.child(e::button().disabled(|ctx: RenderCtx<ToggleAttr<N>>| *ctx.state));
    }

    res
}

const CLASS_ON: Class = class!();
const CLASS_OFF: Class = class!();

#[derive(State, Default)]
struct ToggleClass<const N: u32> {
    state: Signal<bool>,
}

fn render_toggle_class<const N: u32>() -> impl Element<ToggleClass<N>> {
    let mut res = e::div().child(e::button().id(BUTTON).on::<events::Click>(
        |mut ctx: EventCtx<ToggleClass<N>>, _| {
            *ctx.state = !*ctx.state;
        },
    ));

    for _ in 0..N {
        res =
            res.child(e::button().class(
                |ctx: RenderCtx<ToggleClass<N>>| if *ctx.state { CLASS_ON } else { CLASS_OFF },
            ));
    }

    res
}

#[derive(State, Default)]
struct ToggleExist<const N: u32> {
    state: Signal<bool>,
}

fn render_toggle_exist<const N: u32>() -> impl Element<ToggleExist<N>> {
    let mut res = e::div().child(e::button().id(BUTTON).on::<events::Click>(
        |mut ctx: EventCtx<ToggleExist<N>>, _| {
            *ctx.state = !*ctx.state;
        },
    ));

    for _ in 0..N {
        res = res.child(e::div().child(
            |ctx: RenderCtx<ToggleExist<N>>| {
                if *ctx.state { Some("ON") } else { None }
            },
        ));
    }

    res
}

#[derive(State, Default)]
struct ToggleAtOnce<const N: u32> {
    state: Signal<bool>,
}

fn render_toggle_at_once<const N: u32>() -> impl Element<ToggleAtOnce<N>> {
    e::div()
        .child(e::button().id(BUTTON).on::<events::Click>(
            |mut ctx: EventCtx<ToggleAtOnce<N>>, _| {
                *ctx.state = !*ctx.state;
            },
        ))
        .child(|ctx: RenderCtx<ToggleAtOnce<N>>| {
            if *ctx.state {
                let mut res = e::div();
                for _ in 0..N {
                    res = res.child(e::div().text("ON"));
                }
                Some(res)
            } else {
                None
            }
        })
}

macro_rules! define_large_fields {
    ($($field:ident),* $(,)?) => {
        #[derive(State, Default)]
        struct LargeFields {
            $(
                $field: Signal<u32>
            ),*
        }

        fn render_large_fields() -> impl Element<LargeFields> {
            e::div()
                .child(e::button().id(BUTTON2).on::<events::Click>(
                    |mut ctx: EventCtx<LargeFields>, _| {*ctx.a1 += 1;}
                ))
                .child(e::button().id(BUTTON).on::<events::Click>(
                    |mut ctx: EventCtx<LargeFields>, _| {
                        $(
                            *ctx.$field += 1;
                        )*
                    },
                ))
                $(
                    .child(|ctx: RenderCtx<LargeFields>| ctx.$field.clone())
                )*
        }
    };
}

macro_rules! large_fields_matrix {
    ($($suffix:literal),*) => {
        pastey::paste! {
            define_large_fields!(
                $(
                    [< a $suffix >],
                    [< b $suffix >],
                    [< c $suffix >],
                    [< d $suffix >],
                    [< e $suffix >],
                    [< f $suffix >],
                    [< g $suffix >],
                    [< h $suffix >],
                    [< j $suffix >],
                    [< l $suffix >],
                    [< m $suffix >],
                    [< n $suffix >],
                    [< o $suffix >],
                    [< p $suffix >],
                    [< q $suffix >],
                    [< r $suffix >],
                    [< s $suffix >],
                    [< t $suffix >],
                    [< u $suffix >],
                    [< v $suffix >],
                    [< w $suffix >],
                    [< x $suffix >],
                    [< y $suffix >],
                )*
            );
        }
    };
}

large_fields_matrix!(
    1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20, 21, 22, 23, 24, 25, 26, 27, 29,
    30, 31, 32, 33, 34, 35, 36, 37, 39, 40, 41, 42, 43, 44, 45, 46, 47, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 59, 60, 61, 62, 63, 64, 65, 66, 67, 69, 70, 71, 72, 73, 74, 75, 76, 77, 79, 80, 81, 82,
    83, 84, 85, 86, 87, 89, 90, 91, 92, 93, 94, 95, 96, 97, 09, 100
);

#[derive(State, Default)]
struct UpdateNested<const N: u32> {
    state: Signal<u32>,
}

fn render_update_nested<const N: u32>() -> impl Element<UpdateNested<N>> {
    let mut res = e::div().generic();

    for _ in 0..N {
        res = e::button()
            .id(BUTTON)
            .text(|ctx: RenderCtx<UpdateNested<N>>| *ctx.state)
            .on::<events::Click>(|mut ctx: EventCtx<UpdateNested<N>>, _| {
                *ctx.state += 1;
            })
            .child(res)
            .generic();
    }

    res
}

#[derive(State)]
struct DeepStatic<const N: u32>;

fn render_deep_static<const N: u32>() -> impl Element<DeepStatic<N>> {
    let mut res = e::div().generic();

    for _ in 0..N {
        res = e::h1().text("Hey").child(res).generic();
    }

    res
}

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test::wasm_bindgen_test]
async fn run_benchmarks() {
    let mut bencher = runtime::Bencher::new();

    bencher
        .bench("mount_large", 0, |_| {
            natrix::test_utils::mount_test(Buttons::<10000>::default(), render_buttons::<10000>());
        })
        .await;

    natrix::test_utils::mount_test(Buttons::<10000>::default(), render_buttons::<10000>());
    bencher
        .bench("update large", 0, |_| {
            let button = natrix::test_utils::get(BUTTON.0);
            button.click();
        })
        .await;

    natrix::test_utils::mount_test(
        ToggleNode::<10000>::default(),
        render_toggle_node::<10000>(),
    );
    bencher
        .bench("toggle nodes", 0, |_| {
            let button = natrix::test_utils::get(BUTTON.0);
            button.click();
        })
        .await;

    natrix::test_utils::mount_test(
        ToggleText::<10000>::default(),
        render_toggle_text::<10000>(),
    );
    bencher
        .bench("toggle text", 0, |_| {
            let button = natrix::test_utils::get(BUTTON.0);
            button.click();
        })
        .await;

    natrix::test_utils::mount_test(
        ToggleAttr::<10000>::default(),
        render_toggle_attr::<10000>(),
    );
    bencher
        .bench("toggle attribute", 0, |_| {
            let button = natrix::test_utils::get(BUTTON.0);
            button.click();
        })
        .await;

    natrix::test_utils::mount_test(
        ToggleClass::<10000>::default(),
        render_toggle_class::<10000>(),
    );
    bencher
        .bench("toggle class", 0, |_| {
            let button = natrix::test_utils::get(BUTTON.0);
            button.click();
        })
        .await;

    natrix::test_utils::mount_test(
        ToggleExist::<10000>::default(),
        render_toggle_exist::<10000>(),
    );
    bencher
        .bench("toggle exist", 0, |_| {
            let button = natrix::test_utils::get(BUTTON.0);
            button.click();
        })
        .await;

    natrix::test_utils::mount_test(
        ToggleAtOnce::<10000>::default(),
        render_toggle_at_once::<10000>(),
    );
    bencher
        .bench("toggle at once", 0, |_| {
            let button = natrix::test_utils::get(BUTTON.0);
            button.click();
        })
        .await;

    natrix::test_utils::mount_test(LargeFields::default(), render_large_fields());
    bencher
        .bench("update large fields", 0, |_| {
            let button = natrix::test_utils::get(BUTTON.0);
            button.click();
        })
        .await;
    bencher
        .bench("update one on large fields", 0, |_| {
            let button = natrix::test_utils::get(BUTTON2.0);
            button.click();
        })
        .await;

    natrix::test_utils::mount_test(
        UpdateNested::<300>::default(),
        render_update_nested::<300>(),
    );
    bencher
        .bench("update nested", 0, |_| {
            let button = natrix::test_utils::get(BUTTON.0);
            button.click();
        })
        .await;

    bencher
        .bench("deep static", 0, |_| {
            natrix::test_utils::setup();
            natrix::reactivity::mount::mount_at(
                DeepStatic::<1000>,
                render_deep_static::<1000>(),
                natrix::test_utils::MOUNT_POINT,
            )
            .unwrap();
        })
        .await;

    bencher.done();
}
