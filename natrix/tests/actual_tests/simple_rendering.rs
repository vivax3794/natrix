#![allow(dead_code)]

use natrix::prelude::*;
use proptest::proptest;
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

const HELLO_ID: &str = "__HELLO";

#[derive(Component)]
struct HelloWorld;

impl Component for HelloWorld {
    fn render() -> impl Element<Self::Data> {
        e::h1().id(HELLO_ID).text("Hello World!")
    }
}

#[wasm_bindgen_test]
fn renders_fine() {
    crate::setup();
    mount_component(HelloWorld, crate::MOUNT_POINT);

    let element = crate::get(HELLO_ID);
    assert_eq!(element.text_content(), Some("Hello World!".to_owned()));
}

#[derive(Component)]
struct Render<T>(T);

impl<T: Element<Self::Data> + Clone> Component for Render<T> {
    fn render() -> impl Element<Self::Data> {
        e::div().text(|ctx: R<Self>| ctx.0.clone())
    }
}

proptest! {
    #[wasm_bindgen_test]
    fn render_int(x: u32) {
        crate::setup();
        mount_component(Render(x), crate::MOUNT_POINT);
    }
    #[wasm_bindgen_test]
    fn render_float(x: f32) {
        crate::setup();
        mount_component(Render(x), crate::MOUNT_POINT);
    }
    #[wasm_bindgen_test]
    fn render_string(x: String) {
        crate::setup();
        mount_component(Render(x), crate::MOUNT_POINT);
    }
}
