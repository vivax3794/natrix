#![allow(dead_code)]

use natrix::component::NonReactive;
use natrix::prelude::*;
use proptest::proptest;
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

const HELLO_ID: &str = "__HELLO";

#[derive(Component)]
struct HelloWorld;

impl Component for HelloWorld {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::h1().id(HELLO_ID).text("Hello World!")
    }
}

#[wasm_bindgen_test]
fn renders_fine() {
    crate::mount_test(HelloWorld);

    let element = crate::get(HELLO_ID);
    assert_eq!(element.text_content(), Some("Hello World!".to_owned()));
}

#[derive(Component)]
struct Render<T>(T);

impl<T: Element<()> + Clone> Component for Render<T> {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .text(|ctx: R<Self>| NonReactive(ctx.0.clone()))
            .id(HELLO_ID)
    }
}

#[wasm_bindgen_test]
fn render_option_some() {
    crate::mount_test(Render(Some("hey")));

    let element = crate::get(HELLO_ID);
    assert_eq!(element.text_content(), Some("hey".to_owned()));
}

#[wasm_bindgen_test]
fn render_option_none() {
    crate::mount_test(Render(None::<String>));

    let element = crate::get(HELLO_ID);
    assert_eq!(element.text_content(), Some("".to_owned()));
}

#[wasm_bindgen_test]
fn render_result_ok() {
    crate::mount_test(Render(Ok::<&str, &str>("hey")));

    let element = crate::get(HELLO_ID);
    assert_eq!(element.text_content(), Some("hey".to_owned()));
}

#[wasm_bindgen_test]
fn render_result_err() {
    crate::mount_test(Render(Err::<&str, &str>("hey")));

    let element = crate::get(HELLO_ID);
    assert_eq!(element.text_content(), Some("hey".to_owned()));
}

#[cfg(feature = "either")]
mod either_test {
    use either::Either;

    use super::*;

    #[wasm_bindgen_test]
    fn render_either_left() {
        crate::mount_test(Render(Either::Left::<&str, &str>("hey")));

        let element = crate::get(HELLO_ID);
        assert_eq!(element.text_content(), Some("hey".to_owned()));
    }

    #[wasm_bindgen_test]
    fn render_either_right() {
        crate::mount_test(Render(Either::Right::<&str, &str>("hey")));

        let element = crate::get(HELLO_ID);
        assert_eq!(element.text_content(), Some("hey".to_owned()));
    }
}

proptest! {
    #[wasm_bindgen_test]
    fn render_int(x: u32) {

        crate::mount_test(Render(x));
    }
    #[wasm_bindgen_test]
    fn render_float(x: f32) {

        crate::mount_test(Render(x));
    }
    #[wasm_bindgen_test]
    fn render_string(x: String) {

        crate::mount_test(Render(x));
    }
}
