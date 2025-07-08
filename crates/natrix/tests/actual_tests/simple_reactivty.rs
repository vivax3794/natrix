use natrix::format_elements;
use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: Id = natrix::id!();

#[derive(Component)]
struct Counter {
    value: u8,
}

impl natrix::data!(Counter) {
    fn increment(&mut self) {
        *self.value += 1;
    }
}

impl Component for Counter {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::button()
            .id(BUTTON_ID)
            .children(format_elements!(
                |ctx: RenderCtx<Self>| "value: {}-{}",
                *ctx.value,
                *ctx.value + 10
            ))
            .on::<events::Click>(|ctx: Ctx<Self>, _, _| ctx.increment())
    }
}

#[wasm_bindgen_test]
fn renders_initial() {
    crate::mount_test(Counter { value: 0 });

    let button = crate::get(BUTTON_ID);
    assert_eq!(button.text_content(), Some("value: 0-10".to_owned()));
}

#[wasm_bindgen_test]
fn uses_initial_data() {
    crate::mount_test(Counter { value: 123 });

    let button = crate::get(BUTTON_ID);
    assert_eq!(button.text_content(), Some("value: 123-133".to_owned()));
}

#[wasm_bindgen_test]
fn updates_text() {
    crate::mount_test(Counter { value: 0 });

    let button = crate::get(BUTTON_ID);

    button.click();
    assert_eq!(button.text_content(), Some("value: 1-11".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("value: 2-12".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("value: 3-13".to_owned()));
}

#[derive(Component)]
struct TwoValues {
    foo: u8,
    bar: u8,
}

impl Component for TwoValues {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;

    fn render() -> impl Element<Self> {
        e::button()
            .id(BUTTON_ID)
            .text(|ctx: RenderCtx<Self>| format!("{}-{}", *ctx.foo, *ctx.bar))
            .on::<events::Click>(|ctx: Ctx<Self>, _, _| {
                *ctx.foo += 1;
            })
    }
}

#[wasm_bindgen_test]
fn test_two_values() {
    crate::mount_test(TwoValues { foo: 0, bar: 0 });

    let button = crate::get(BUTTON_ID);

    assert_eq!(button.text_content(), Some("0-0".to_string()));

    button.click();
    assert_eq!(button.text_content(), Some("1-0".to_string()));
}
