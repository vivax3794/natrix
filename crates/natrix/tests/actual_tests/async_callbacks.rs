#![cfg(feature = "async_utils")]

use std::time::Duration;

use natrix::async_utils;
use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: Id = natrix::id!();

#[derive(State)]
struct AsyncComponent {
    data: Signal<u8>,
}

fn render_async_component() -> impl Element<AsyncComponent> {
    e::button()
        .id(BUTTON_ID)
        .text(|ctx: &mut RenderCtx<AsyncComponent>| *ctx.data)
        .on::<events::Click>(|mut ctx: EventCtx<AsyncComponent>, _| {
            ctx.use_async(async |ctx| {
                async_utils::sleep_milliseconds(10).await;
                ctx.update(|mut ctx| {
                    *ctx.data += 10;
                })?;
                Some(())
            });
        })
}

#[wasm_bindgen_test]
async fn async_works() {
    crate::mount_test(
        AsyncComponent {
            data: Signal::new(0),
        },
        render_async_component(),
    );

    let button = crate::get(BUTTON_ID);

    button.click();
    async_utils::sleep_milliseconds(20).await;
    assert_eq!(button.text_content(), Some("10".to_owned()));
}

#[wasm_bindgen_test]
async fn async_multiple() {
    crate::mount_test(
        AsyncComponent {
            data: Signal::new(0),
        },
        render_async_component(),
    );

    let button = crate::get(BUTTON_ID);

    button.click();
    button.click();
    button.click();
    async_utils::sleep_milliseconds(30).await;
    assert_eq!(button.text_content(), Some("30".to_owned()));
}
