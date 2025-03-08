#![allow(dead_code)]

use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: &str = "__BUTTON";
const TEXT: &str = "__TEXT";

#[derive(Component)]
struct Counter {
    value: u8,
}

impl Component for Counter {
    fn render() -> impl Element<Self::Data> {
        e::button()
            .id(BUTTON_ID)
            .child(|mut ctx: R<Self>| {
                if ctx.watch(|ctx| *ctx.value > 2) {
                    e::div().text(|ctx: R<Self>| *ctx.value).id(TEXT)
                } else {
                    e::div()
                }
            })
            .on::<events::Click>(|ctx: &mut S<Self>, _| *ctx.value += 1)
    }
}

#[wasm_bindgen_test]
fn works() {
    crate::setup();
    mount_component(Counter { value: 0 }, crate::MOUNT_POINT);

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
