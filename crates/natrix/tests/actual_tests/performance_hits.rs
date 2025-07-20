// NOTE: These tests were written to trigger `performance_lint`s.
// As such they might seem a bit nonsensical.

use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON: Id = natrix::id!();

#[derive(State, Default)]
struct StaleDepAccumulation {
    modified: Signal<u8>,
    read_only: Signal<u8>,
}

fn render_stale_dep() -> impl Element<StaleDepAccumulation> {
    e::div()
        .child(
            e::button()
                .id(BUTTON)
                .on::<events::Click>(|mut ctx: EventCtx<StaleDepAccumulation>, _| {
                    *ctx.modified += 1;
                })
                .text(|ctx: RenderCtx<StaleDepAccumulation>| *ctx.modified),
        )
        .child(|ctx: RenderCtx<StaleDepAccumulation>| {
            *ctx.modified;
            |ctx: RenderCtx<StaleDepAccumulation>| *ctx.read_only
        })
}

// As of writing this causes the `read_only` dep list to grow without ever being
// cleared
#[wasm_bindgen_test]
fn stale_dep_accumulation() {
    crate::mount_test(StaleDepAccumulation::default(), render_stale_dep());
    let button = crate::get(BUTTON);

    for _ in 0..50 {
        button.click();
    }

    let text = button.text_content();
    assert_eq!(text, Some("50".to_string()));
}
