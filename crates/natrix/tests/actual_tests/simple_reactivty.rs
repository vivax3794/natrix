use natrix::format_elements;
use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: Id = natrix::id!();

#[derive(State)]
struct Counter {
    value: Signal<u8>,
}

impl Counter {
    fn increment(&mut self) {
        *self.value += 1;
    }
}

fn render_counter() -> impl Element<Counter> {
    e::button()
        .id(BUTTON_ID)
        .children(format_elements!(
            |ctx: RenderCtx<Counter>| "value: {}-{}",
            *ctx.value,
            *ctx.value + 10
        ))
        .on::<events::Click>(|mut ctx: EventCtx<Counter>, _| ctx.increment())
}

#[wasm_bindgen_test]
fn renders_initial() {
    crate::mount_test(
        Counter {
            value: Signal::new(0),
        },
        render_counter(),
    );

    let button = crate::get(BUTTON_ID);
    assert_eq!(button.text_content(), Some("value: 0-10".to_owned()));
}

#[wasm_bindgen_test]
fn uses_initial_data() {
    crate::mount_test(
        Counter {
            value: Signal::new(123),
        },
        render_counter(),
    );

    let button = crate::get(BUTTON_ID);
    assert_eq!(button.text_content(), Some("value: 123-133".to_owned()));
}

#[wasm_bindgen_test]
fn updates_text() {
    crate::mount_test(
        Counter {
            value: Signal::new(0),
        },
        render_counter(),
    );

    let button = crate::get(BUTTON_ID);

    button.click();
    assert_eq!(button.text_content(), Some("value: 1-11".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("value: 2-12".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("value: 3-13".to_owned()));
}

#[derive(State)]
struct TwoValues {
    foo: Signal<u8>,
    bar: Signal<u8>,
}

fn render_two() -> impl Element<TwoValues> {
    e::button()
        .id(BUTTON_ID)
        .text(|ctx: RenderCtx<TwoValues>| format!("{}-{}", *ctx.foo, *ctx.bar))
        .on::<events::Click>(|mut ctx: EventCtx<TwoValues>, _| {
            *ctx.foo += 1;
        })
}

#[wasm_bindgen_test]
fn test_two_values() {
    crate::mount_test(
        TwoValues {
            foo: Signal::new(0),
            bar: Signal::new(0),
        },
        render_two(),
    );

    let button = crate::get(BUTTON_ID);

    assert_eq!(button.text_content(), Some("0-0".to_string()));

    button.click();
    assert_eq!(button.text_content(), Some("1-0".to_string()));
}
