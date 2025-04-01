#![allow(dead_code)]

use natrix::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

const ROOT_ID: &str = "__HELLO";

#[derive(Component, Default)]
struct Generic<T>(T);

impl<T: Element<Self::Data> + Copy> Component for Generic<T> {
    fn render() -> impl Element<Self::Data> {
        e::div().id(ROOT_ID).text(|ctx: R<Self>| *ctx.0)
    }
}

#[wasm_bindgen_test]
fn generic_int() {
    crate::mount_test(Generic::<u8>::default());

    let element = crate::get(ROOT_ID);
    assert_eq!(element.text_content(), Some("0".to_owned()));
}

#[wasm_bindgen_test]
fn generic_str() {
    crate::mount_test(Generic("Hello World"));

    let element = crate::get(ROOT_ID);
    assert_eq!(element.text_content(), Some("Hello World".to_owned()));
}
