#![cfg(feature = "async_utils")]

use std::time::Duration;

use natrix::async_utils;
use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: &str = "__BUTTON";

#[derive(Component)]
struct AsyncComponent {
    data: u8,
}

impl Component for AsyncComponent {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::button()
            .id(BUTTON_ID)
            .text(|ctx: R<Self>| *ctx.data)
            .on::<events::Click>(|ctx: E<Self>, _| {
                ctx.use_async(async |mut ctx| {
                    async_utils::sleep(Duration::from_millis(10)).await;
                    let mut borrow = ctx.borrow_mut()?;
                    *borrow.data += 10;
                    Some(())
                });
            })
    }
}

#[wasm_bindgen_test]
async fn async_works() {
    crate::mount_test(AsyncComponent { data: 0 });

    let button = crate::get(BUTTON_ID);

    button.click();
    async_utils::sleep(Duration::from_millis(20)).await;
    assert_eq!(button.text_content(), Some("10".to_owned()));
}

#[wasm_bindgen_test]
async fn async_multiple() {
    crate::mount_test(AsyncComponent { data: 0 });

    let button = crate::get(BUTTON_ID);

    button.click();
    button.click();
    button.click();
    async_utils::sleep(Duration::from_millis(30)).await;
    assert_eq!(button.text_content(), Some("30".to_owned()));
}
