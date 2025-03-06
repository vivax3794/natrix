#![allow(dead_code)]

use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: &str = "__BUTTON";

#[derive(Component)]
struct Counter {
    value: u8,
}

#[reactive]
impl Counter {
    fn larger_than(&self, other: u8) -> bool {
        self.value > other
    }
}

impl Component for Counter {
    fn render() -> impl Element<Self::Data> {
        e::button()
            .id(BUTTON_ID)
            .text(
                |ctx: &S<Self>| {
                    if ctx.larger_than(2) { "Hello" } else { "World" }
                },
            )
            .on("click", |ctx: &mut S<Self>| *ctx.value += 1)
    }
}

#[wasm_bindgen_test]
fn one_value() {
    crate::setup();
    mount_component(Counter { value: 0 }, crate::MOUNT_POINT);

    let button = crate::get(BUTTON_ID);

    assert_eq!(button.text_content(), Some("World".to_owned()));
    button.click();
    button.click();
    button.click();
    assert_eq!(button.text_content(), Some("Hello".to_owned()));
}
