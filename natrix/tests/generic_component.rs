#![allow(dead_code)]

use natrix::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

mod common;

const ROOT_ID: &str = "__HELLO";

#[derive(Component, Default)]
struct Generic<T>(T);

impl<T: Element<Self::Data> + Copy> Component for Generic<T> {
    fn render() -> impl Element<Self::Data> {
        e::div().id(ROOT_ID).text(|ctx: &S<Self>| *ctx.0)
    }
}

#[wasm_bindgen_test]
fn generic_int() {
    common::setup();
    mount_component(Generic::<u8>::default(), common::MOUNT_POINT);

    let element = common::get(ROOT_ID);
    assert_eq!(element.text_content(), Some("0".to_owned()));
}

#[wasm_bindgen_test]
fn generic_str() {
    common::setup();
    mount_component(Generic("Hello World"), common::MOUNT_POINT);

    let element = common::get(ROOT_ID);
    assert_eq!(element.text_content(), Some("Hello World".to_owned()));
}
