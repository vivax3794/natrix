use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_1: Id = natrix::id!();
const BUTTON_2: Id = natrix::id!();
const TEXT: Id = natrix::id!();

#[derive(State, Default)]
struct DoubleCounter {
    value_one: Signal<u8>,
    value_two: Signal<u8>,
}

fn render_double_counter() -> impl Element<DoubleCounter> {
    e::div()
        .child(e::button().id(BUTTON_1).on::<events::Click>(
            |mut ctx: EventCtx<DoubleCounter>, _, _| {
                *ctx.value_one += 1;
            },
        ))
        .child(e::button().id(BUTTON_2).on::<events::Click>(
            |mut ctx: EventCtx<DoubleCounter>, _, _| {
                *ctx.value_two += 1;
            },
        ))
        .child(|ctx: &mut RenderCtx<DoubleCounter>| {
            (*ctx.value_one >= 2).then_some(
                e::div()
                    .id(TEXT)
                    .child(|ctx: &mut RenderCtx<DoubleCounter>| *ctx.value_two),
            )
        })
}

#[wasm_bindgen_test]
fn update_affects_inner_node() {
    crate::mount_test(DoubleCounter::default(), render_double_counter());

    let button_1 = crate::get(BUTTON_1);
    let button_2 = crate::get(BUTTON_2);

    button_1.click();
    button_1.click();
    button_1.click();
    button_1.click();

    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("0".to_owned()));

    button_2.click();
    assert_eq!(text.text_content(), Some("1".to_owned()));

    button_2.click();
    assert_eq!(text.text_content(), Some("2".to_owned()));

    button_2.click();
    assert_eq!(text.text_content(), Some("3".to_owned()));

    button_2.click();
    assert_eq!(text.text_content(), Some("4".to_owned()));

    button_2.click();
    assert_eq!(text.text_content(), Some("5".to_owned()));
}
