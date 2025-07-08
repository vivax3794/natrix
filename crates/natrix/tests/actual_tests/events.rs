use natrix::prelude::*;
use natrix::reactivity::EventToken;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: Id = natrix::id!();

#[derive(Component)]
struct Counter {
    value: u8,
}

impl Component for Counter {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::button()
            .id(BUTTON_ID)
            .child(|ctx: RenderCtx<Self>| *ctx.value)
            .on::<events::Click>(|ctx: Ctx<Self>, _, _| *ctx.value += 1)
    }
}

#[wasm_bindgen_test]
fn can_use_event() {
    crate::mount_test(Counter { value: 0 });

    let button = crate::get(BUTTON_ID);

    button.click();
    assert_eq!(button.text_content(), Some("1".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("2".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("3".to_owned()));
}

#[derive(Component)]
struct OnMount {
    value: u8,
}

impl Component for OnMount {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .id(BUTTON_ID)
            .text(|ctx: RenderCtx<Self>| *ctx.value)
    }
    fn on_mount(ctx: Ctx<Self>, _token: EventToken) {
        *ctx.value = 10;
    }
}

#[wasm_bindgen_test]
fn on_mount() {
    crate::mount_test(OnMount { value: 0 });

    let text = crate::get(BUTTON_ID);
    assert_eq!(text.text_content(), Some("10".to_string()));
}
