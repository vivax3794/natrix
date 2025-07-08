use natrix::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

const ROOT_ID: Id = natrix::id!();

#[derive(Component, Default)]
struct Generic<T>(T);

impl<T: ToString + 'static> Component for Generic<T> {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .id(ROOT_ID)
            .text(|ctx: RenderCtx<Self>| ctx.0.to_string())
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
