#![cfg(feature = "async")]

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
    fn render() -> impl Element<Self::Data> {
        e::button()
            .id(BUTTON_ID)
            .text(|ctx: &S<Self>| *ctx.data)
            .on("click", |ctx: &mut S<Self>| {
                ctx.use_async(async |mut ctx| {
                    async_utils::sleep(Duration::from_millis(200)).await;
                    *ctx.borrow_mut().unwrap().data += 10;
                });
            })
    }
}

#[wasm_bindgen_test]
async fn async_works() {
    crate::setup();
    mount_component(AsyncComponent { data: 0 }, crate::MOUNT_POINT);

    let button = crate::get(BUTTON_ID);

    button.click();
    async_utils::sleep(Duration::from_millis(300)).await;
    assert_eq!(button.text_content(), Some("10".to_owned()));
}

#[wasm_bindgen_test]
async fn async_multiple() {
    crate::setup();
    mount_component(AsyncComponent { data: 0 }, crate::MOUNT_POINT);

    let button = crate::get(BUTTON_ID);

    button.click();
    button.click();
    button.click();
    async_utils::sleep(Duration::from_millis(400)).await;
    assert_eq!(button.text_content(), Some("30".to_owned()));
}
