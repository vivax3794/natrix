// NOTE: These tests were written to trigger `performance_lint`s.
// As such they might seem a bit nonsensical.

use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON: Id = natrix::id!();

#[derive(Component, Default)]
struct StaleDepAccumulation {
    modified: u8,
    read_only: u8,
}

impl Component for StaleDepAccumulation {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;

    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: Ctx<Self>, _, _| {
                        *ctx.modified += 1;
                    })
                    .text(|ctx: RenderCtx<Self>| *ctx.modified),
            )
            .child(|ctx: RenderCtx<Self>| {
                *ctx.modified;
                |ctx: RenderCtx<Self>| *ctx.read_only
            })
    }
}

// As of writing this causes the `read_only` dep list to grow without ever being
// cleared
#[wasm_bindgen_test]
#[ignore = "Unsure whether we want to optimize this"]
fn stale_dep_accumulation() {
    crate::mount_test(StaleDepAccumulation::default());
    let button = crate::get(BUTTON);

    for _ in 0..50 {
        button.click();
    }

    let text = button.text_content();
    assert_eq!(text, Some("50".to_string()));
}
