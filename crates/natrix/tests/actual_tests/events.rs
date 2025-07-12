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
        .child(|ctx: &mut RenderCtx<Counter>| *ctx.value)
        .on::<events::Click>(|mut ctx: EventCtx<Counter>, _| ctx.increment())
}

#[wasm_bindgen_test]
fn can_use_event() {
    crate::mount_test(
        Counter {
            value: Signal::new(0),
        },
        render_counter(),
    );

    let button = crate::get(BUTTON_ID);

    button.click();
    assert_eq!(button.text_content(), Some("1".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("2".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("3".to_owned()));
}
