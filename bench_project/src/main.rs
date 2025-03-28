use natrix::prelude::*;
use wasm_bench_runtime::Bencher;

#[derive(Component)]
struct LargeDom<const N: u32>;

impl<const N: u32> Component for LargeDom<N> {
    fn render() -> impl Element<Self::Data> {
        let mut res = e::div();
        for _ in 0..N {
            res = res.child(e::h1().text("SUCH LARGE"));
        }
        res
    }
}

#[derive(Component)]
struct DeepDom<const N: u32>;

impl<const N: u32> Component for DeepDom<N> {
    fn render() -> impl Element<Self::Data> {
        let mut res = e::div();
        for _ in 0..N {
            res = e::div().text("SUCH DEEP").child(res);
        }
        res
    }
}

#[derive(Component)]
struct ManyButtons<const N: u32> {
    counter: u32,
}

impl<const N: u32> Component for ManyButtons<N> {
    fn render() -> impl Element<Self::Data> {
        let mut res = e::div();
        for _ in 0..N {
            res = res.child(
                e::button()
                    .id("BUTTON")
                    .text(|ctx: R<Self>| *ctx.counter)
                    .on::<events::Click>(|ctx: &mut S<Self>, _| {
                        *ctx.counter += 1;
                    }),
            )
        }
        res
    }
}

fn main() {
    Bencher::start(async |mut bencher| {
        bencher
            .bench("inital_dom_50k", 0, |_| {
                natrix::test_utils::setup();
                mount_component(LargeDom::<50_000>, natrix::test_utils::MOUNT_POINT);
            })
            .await;
        bencher
            .bench("deep_dom_100", 0, |_| {
                natrix::test_utils::setup();
                mount_component(DeepDom::<100>, natrix::test_utils::MOUNT_POINT);
            })
            .await;
        bencher
            .bench("mount_events_10k", 0, |_| {
                natrix::test_utils::setup();
                mount_component(
                    ManyButtons::<10_000> { counter: 0 },
                    natrix::test_utils::MOUNT_POINT,
                );
            })
            .await;

        natrix::test_utils::setup();
        mount_component(
            ManyButtons::<10_000> { counter: 0 },
            natrix::test_utils::MOUNT_POINT,
        );
        let button = natrix::test_utils::get("BUTTON");
        bencher
            .bench("text_updates_10k", 0, |_| {
                button.click();
            })
            .await;
    });
}
