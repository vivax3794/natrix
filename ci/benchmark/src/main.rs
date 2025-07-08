use natrix::class;
use natrix::prelude::*;
use wasm_bench_runtime::Bencher;

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
    a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y
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

fn main() {
    Bencher::start(async |mut bencher| {
        bencher
            .bench("mount_large", 0, |_| {
                natrix::test_utils::mount_test(
                    Buttons::<10000>::default(),
                    render_buttons::<10000>(),
                );
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
    });
}
