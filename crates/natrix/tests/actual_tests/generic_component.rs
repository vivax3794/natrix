use natrix::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

const ROOT_ID: Id = natrix::id!();

#[derive(State, Default)]
struct Generic<T>(Signal<T>);

fn render_generic<T: ToString + 'static>() -> impl Element<Generic<T>> {
    e::div()
        .id(ROOT_ID)
        .text(|ctx: RenderCtx<Generic<T>>| ctx.0.to_string())
}

#[wasm_bindgen_test]
fn generic_int() {
    crate::mount_test(Generic(Signal::new(0u8)), render_generic::<u8>());

    let element = crate::get(ROOT_ID);
    assert_eq!(element.text_content(), Some("0".to_owned()));
}

#[wasm_bindgen_test]
fn generic_str() {
    crate::mount_test(
        Generic(Signal::new("Hello World")),
        render_generic::<&str>(),
    );

    let element = crate::get(ROOT_ID);
    assert_eq!(element.text_content(), Some("Hello World".to_owned()));
}
