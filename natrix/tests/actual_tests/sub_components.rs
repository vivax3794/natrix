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
    type EmitMessage = u8;
    fn render() -> impl Element<Self> {
        e::button()
            .id(BUTTON_ID)
            .text(|ctx: R<Self>| *ctx.value)
            .on::<events::Click>(|ctx: &mut S<Self>, _| {
                *ctx.value += 1;
                ctx.emit(*ctx.value);
            })
    }
}

const DOUBLE_ID: &str = "DOUBLE_ID";

#[derive(Component)]
struct RootOne {
    double: u8,
}

impl Component for RootOne {
    type EmitMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                C::new(Counter { value: 0 }).on(|ctx: &mut S<Self>, amount| {
                    *ctx.double = amount * 2;
                }),
            )
            .child(e::div().id(DOUBLE_ID).text(|ctx: R<Self>| *ctx.double))
    }
}

#[wasm_bindgen_test]
fn simple_button_child() {
    crate::mount_test(RootOne { double: 0 });

    let button = crate::get(BUTTON_ID);

    assert_eq!(button.text_content(), Some("0".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("1".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("2".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("3".to_owned()));
}

#[wasm_bindgen_test]
#[cfg(feature = "async_utils")]
async fn child_to_parent() {
    use natrix::async_utils;
    crate::mount_test(RootOne { double: 0 });

    let button = crate::get(BUTTON_ID);
    let double = crate::get(DOUBLE_ID);

    assert_eq!(button.text_content(), Some("0".to_owned()));
    assert_eq!(double.text_content(), Some("0".to_owned()));

    button.click();
    async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("1".to_owned()));
    assert_eq!(double.text_content(), Some("2".to_owned()));

    button.click();
    async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("2".to_owned()));
    assert_eq!(double.text_content(), Some("4".to_owned()));

    button.click();
    async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("3".to_owned()));
    assert_eq!(double.text_content(), Some("6".to_owned()));
}
