use natrix::prelude::*;
use wasm_bench_runtime::Bencher;

#[derive(Component, Default)]
struct Buttons<const N: u32> {
    state: u32,
}

impl<const N: u32> Component for Buttons<N> {
    fn render() -> impl Element<Self> {
        let mut res = e::div();

        for _ in 0..N {
            res = res.child(
                e::button()
                    .id("BUTTON")
                    .text(|ctx: R<Self>| *ctx.state)
                    .on::<events::Click>(|ctx: E<Self>, _, _| {
                        *ctx.state += 1;
                    }),
            );
        }

        res
    }
}

#[derive(Component, Default)]
struct ToggleNode<const N: u32> {
    state: bool,
}

impl<const N: u32> Component for ToggleNode<N> {
    fn render() -> impl Element<Self> {
        let mut res = e::div().child(e::button().id("BUTTON").on::<events::Click>(
            |ctx: E<Self>, _, _| {
                *ctx.state = !*ctx.state;
            },
        ));

        for _ in 0..N {
            res = res.child(e::div().child(|ctx: R<Self>| {
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
}

#[derive(Component, Default)]
struct ToggleText<const N: u32> {
    state: bool,
}

impl<const N: u32> Component for ToggleText<N> {
    fn render() -> impl Element<Self> {
        let mut res = e::div().child(e::button().id("BUTTON").on::<events::Click>(
            |ctx: E<Self>, _, _| {
                *ctx.state = !*ctx.state;
            },
        ));

        for _ in 0..N {
            res = res.child(e::div().child(|ctx: R<Self>| if *ctx.state { "ON" } else { "OFF" }));
        }

        res
    }
}

#[derive(Component, Default)]
struct ToggleExist<const N: u32> {
    state: bool,
}

impl<const N: u32> Component for ToggleExist<N> {
    fn render() -> impl Element<Self> {
        let mut res = e::div().child(e::button().id("BUTTON").on::<events::Click>(
            |ctx: E<Self>, _, _| {
                *ctx.state = !*ctx.state;
            },
        ));

        for _ in 0..N {
            res = res.child(e::div().child(
                |ctx: R<Self>| {
                    if *ctx.state { Some("ON") } else { None }
                },
            ));
        }

        res
    }
}

#[derive(Component, Default)]
struct ToggleAtOnce<const N: u32> {
    state: bool,
}

impl<const N: u32> Component for ToggleAtOnce<N> {
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::button()
                    .id("BUTTON")
                    .on::<events::Click>(|ctx: E<Self>, _, _| {
                        *ctx.state = !*ctx.state;
                    }),
            )
            .child(|ctx: R<Self>| {
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
}

macro_rules! define_large_fields {
    ($($field:ident),*) => {
        #[derive(Component, Default)]
        struct LargeFields {
            $(
                $field: u32
            ),*
        }

        impl Component for LargeFields {
            fn render() -> impl Element<Self> {
                e::div()
                    .child(e::button().id("BUTTON").on::<events::Click>(
                        |ctx: E<Self>, _, _| {
                            $(
                                *ctx.$field += 1;
                            )*
                        },
                    ))
                    $(
                        .child(|ctx: R<Self>| ctx.$field.clone())
                    )*
            }
        }
    };
}
define_large_fields!(
    a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y
);

#[derive(Component, Default)]
struct UpdateNested<const N: u32> {
    state: u32,
}

impl<const N: u32> Component for UpdateNested<N> {
    fn render() -> impl Element<Self> {
        let mut res = e::div().generic();

        for _ in 0..N {
            res = e::button()
                .id("BUTTON")
                .text(|ctx: R<Self>| *ctx.state)
                .on::<events::Click>(|ctx: E<Self>, _, _| {
                    *ctx.state += 1;
                })
                .child(res)
                .generic();
        }

        res
    }
}

#[derive(Component)]
struct DeepStatic<const N: u32>;

impl<const N: u32> Component for DeepStatic<N> {
    fn render() -> impl Element<Self> {
        let mut res = e::div().generic();

        for _ in 0..N {
            res = e::h1().text("Hey").child(res).generic();
        }

        res
    }
}

fn main() {
    Bencher::start(async |mut bencher| {
        bencher
            .bench("mount_large", 0, |_| {
                // WARNING: This does include the `mount_test` cleaning up the previous dom tree.
                // But at least on the rust side that should be minimal in comparison to the
                // mounting.
                natrix::test_utils::setup();
                natrix::reactivity::component::mount_at(
                    Buttons::<10000>::default(),
                    natrix::test_utils::MOUNT_POINT,
                )
                .unwrap();
            })
            .await;

        natrix::test_utils::mount_test(Buttons::<10000>::default());
        bencher
            .bench("update large", 0, |_| {
                let button = natrix::test_utils::get("BUTTON");
                button.click();
            })
            .await;

        natrix::test_utils::mount_test(ToggleNode::<10000>::default());
        bencher
            .bench("toggle nodes", 0, |_| {
                let button = natrix::test_utils::get("BUTTON");
                button.click();
            })
            .await;

        natrix::test_utils::mount_test(ToggleText::<10000>::default());
        bencher
            .bench("toggle text", 0, |_| {
                let button = natrix::test_utils::get("BUTTON");
                button.click();
            })
            .await;

        natrix::test_utils::mount_test(ToggleExist::<10000>::default());
        bencher
            .bench("toggle exist", 0, |_| {
                let button = natrix::test_utils::get("BUTTON");
                button.click();
            })
            .await;

        natrix::test_utils::mount_test(ToggleAtOnce::<10000>::default());
        bencher
            .bench("toggle at once", 0, |_| {
                let button = natrix::test_utils::get("BUTTON");
                button.click();
            })
            .await;

        natrix::test_utils::mount_test(LargeFields::default());
        bencher
            .bench("update large fields", 0, |_| {
                let button = natrix::test_utils::get("BUTTON");
                button.click();
            })
            .await;

        natrix::test_utils::mount_test(UpdateNested::<100>::default());
        bencher
            .bench("update nested", 0, |_| {
                let button = natrix::test_utils::get("BUTTON");
                button.click();
            })
            .await;

        bencher
            .bench("deep static", 0, |_| {
                natrix::test_utils::mount_test(DeepStatic::<1000>);
            })
            .await;
    });
}
