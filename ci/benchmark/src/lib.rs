#![allow(dead_code, reason = "test")]

use natrix::class;
use natrix::prelude::*;

mod runtime;

const BUTTON: Id = natrix::id!();

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
                .text(|ctx: &mut RenderCtx<Buttons<N>>| *ctx.state)
                .on::<events::Click>(|ctx: &mut Ctx<Buttons<N>>, _, _| {
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
        |ctx: &mut Ctx<ToggleNode<N>>, _, _| {
            *ctx.state = !*ctx.state;
        },
    ));

    for _ in 0..N {
        res = res.child(e::div().child(|ctx: &mut RenderCtx<ToggleNode<N>>| {
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
        |ctx: &mut Ctx<ToggleText<N>>, _, _| {
            *ctx.state = !*ctx.state;
        },
    ));

    for _ in 0..N {
        res = res.child(
            e::div()
                .child(|ctx: &mut RenderCtx<ToggleText<N>>| if *ctx.state { "ON" } else { "OFF" }),
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
        |ctx: &mut Ctx<ToggleAttr<N>>, _, _| {
            *ctx.state = !*ctx.state;
        },
    ));

    for _ in 0..N {
        res = res.child(e::button().disabled(|ctx: &mut RenderCtx<ToggleAttr<N>>| *ctx.state));
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
        |ctx: &mut Ctx<ToggleClass<N>>, _, _| {
            *ctx.state = !*ctx.state;
        },
    ));

    for _ in 0..N {
        res = res.child(e::button().class(
            |ctx: &mut RenderCtx<ToggleClass<N>>| if *ctx.state { CLASS_ON } else { CLASS_OFF },
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
        |ctx: &mut Ctx<ToggleExist<N>>, _, _| {
            *ctx.state = !*ctx.state;
        },
    ));

    for _ in 0..N {
        res = res.child(e::div().child(
            |ctx: &mut RenderCtx<ToggleExist<N>>| {
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
            |ctx: &mut Ctx<ToggleAtOnce<N>>, _, _| {
                *ctx.state = !*ctx.state;
            },
        ))
        .child(|ctx: &mut RenderCtx<ToggleAtOnce<N>>| {
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
    ($($field:ident),*) => {
        #[derive(State, Default)]
        struct LargeFields {
            $(
                $field: Signal<u32>
            ),*
        }

        fn render_large_fields() -> impl Element<LargeFields> {
            e::div()
                .child(e::button().id(BUTTON).on::<events::Click>(
                    |ctx: &mut Ctx<LargeFields>, _, _| {
                        $(
                            *ctx.$field += 1;
                        )*
                    },
                ))
                $(
                    .child(|ctx: &mut RenderCtx<LargeFields>| ctx.$field.clone())
                )*
        }
    };
}
define_large_fields!(
    a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, a1, b1, c1, d1, e1,
    f1, g1, h1, i1, j1, k1, l1, m1, n1, o1, p1, q1, r1, s1, t1, u1, v1, w1, x1, y1, a2, b2, c2, d2,
    e2, f2, g2, h2, i2, j2, k2, l2, m2, n2, o2, p2, q2, r2, s2, t2, u2, v2, w2, x2, y2, a3, b3, c3,
    d3, e3, f3, g3, h3, i3, j3, k3, l3, m3, n3, o3, p3, q3, r3, s3, t3, u3, v3, w3, x3, y3, a4, b4,
    c4, d4, e4, f4, g4, h4, i4, j4, k4, l4, m4, n4, o4, p4, q4, r4, s4, t4, u4, v4, w4, x4, y4, a5,
    b5, c5, d5, e5, f5, g5, h5, i5, j5, k5, l5, m5, n5, o5, p5, q5, r5, s5, t5, u5, v5, w5, x5, y5,
    a6, b6, c6, d6, e6, f6, g6, h6, i6, j6, k6, l6, m6, n6, o6, p6, q6, r6, s6, t6, u6, v6, w6, x6,
    y6, a7, b7, c7, d7, e7, f7, g7, h7, i7, j7, k7, l7, m7, n7, o7, p7, q7, r7, s7, t7, u7, v7, w7,
    x7, y7, a8, b8, c8, d8, e8, f8, g8, h8, i8, j8, k8, l8, m8, n8, o8, p8, q8, r8, s8, t8, u8, v8,
    w8, x8, y8, a9, b9, c9, d9, e9, f9, g9, h9, i9, j9, k9, l9, m9, n9, o9, p9, q9, r9, s9, t9, u9,
    v9, w9, x9, y9
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
            .text(|ctx: &mut RenderCtx<UpdateNested<N>>| *ctx.state)
            .on::<events::Click>(|ctx: &mut Ctx<UpdateNested<N>>, _, _| {
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
