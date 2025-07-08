#![cfg(false)]

use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: Id = natrix::id!();
const TEXT: Id = natrix::id!();

#[derive(State)]
struct Counter {
    value: Signal<u8>,
}

impl Counter {
    fn increment(&mut self) {
        *self.value += 1;
    }
}

fn render_counter() -> impl Element<Counter> {
    e::button()
        .id(BUTTON_ID)
        .child(|ctx: &mut RenderCtx<Counter>| {
            if ctx.watch(|ctx| *ctx.value > 2) {
                e::div()
                    .text(|ctx: &mut RenderCtx<Counter>| *ctx.value)
                    .id(TEXT)
            } else {
                e::div()
            }
        })
        .on::<events::Click>(|ctx: &mut Ctx<Counter>, _, _| ctx.increment())
}

#[wasm_bindgen_test]
fn works() {
    crate::mount_test(Counter { value: 0 }, render_counter());

    let button = crate::get(BUTTON_ID);

    button.click();
    button.click();
    button.click();

    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("3".to_owned()));

    button.click();
    assert_eq!(text.text_content(), Some("4".to_owned()));

    button.click();
    assert_eq!(text.text_content(), Some("5".to_owned()));
}
