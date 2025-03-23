#![allow(dead_code)]

use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: &str = "BUTTON";
const BUTTON_TWO: &str = "BUTTON_TWO";

#[derive(Component)]
struct Counter {
    value: u8,
}

impl Component for Counter {
    fn render() -> impl Element<Self::Data> {
        e::button()
            .id(BUTTON_ID)
            .text(|ctx: R<Self>| *ctx.value)
            .on::<events::Click>(|ctx: &mut S<Self>, _| {
                *ctx.value += 1;
            })
    }
}

#[derive(Component)]
struct RootOne;

impl Component for RootOne {
    fn render() -> impl Element<Self::Data> {
        e::div().child(C(Counter { value: 0 }))
    }
}

#[wasm_bindgen_test]
fn simple_button_child() {
    crate::setup();
    mount_component(RootOne, crate::MOUNT_POINT);

    let button = crate::get(BUTTON_ID);

    assert_eq!(button.text_content(), Some("0".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("1".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("2".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("3".to_owned()));
}
