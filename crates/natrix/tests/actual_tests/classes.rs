use natrix::prelude::*;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: Id = natrix::id!();
const DECREMENT_ID: Id = natrix::id!();
const CLICKED_CLASS: Class = natrix::class!();
const NOT_CLICKED_CLASS: Class = natrix::class!();
const CLICKED_MORE_THAN_2_CLASS: Class = natrix::class!();

#[derive(Component)]
struct HelloWorld {
    counter: usize,
}

impl Component for HelloWorld {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON_ID)
                    .on::<events::Click>(|ctx: E<Self>, _, _| {
                        *ctx.counter += 1;
                    })
                    .class(|ctx: R<Self>| {
                        if *ctx.counter > 0 {
                            CLICKED_CLASS
                        } else {
                            NOT_CLICKED_CLASS
                        }
                    })
                    .class(|ctx: R<Self>| {
                        if *ctx.counter > 2 {
                            Some(CLICKED_MORE_THAN_2_CLASS)
                        } else {
                            None
                        }
                    }),
            )
            .child(
                e::button()
                    .id(DECREMENT_ID)
                    .on::<events::Click>(|ctx: E<Self>, _, _| {
                        *ctx.counter -= 1;
                    }),
            )
    }
}

#[wasm_bindgen_test]
fn test_class_initial() {
    crate::mount_test(HelloWorld { counter: 0 });
    let button = crate::get(BUTTON_ID);

    assert_eq!(button.class_name(), NOT_CLICKED_CLASS.0);
}

#[wasm_bindgen_test]
fn test_class_pure_str_change() {
    crate::mount_test(HelloWorld { counter: 0 });
    let button = crate::get(BUTTON_ID);

    button.click();
    assert_eq!(button.class_name(), CLICKED_CLASS.0);
}

#[wasm_bindgen_test]
fn test_option() {
    crate::mount_test(HelloWorld { counter: 0 });
    let button = crate::get(BUTTON_ID);

    button.click();
    button.click();
    button.click();
    assert!(button.class_name().contains(CLICKED_MORE_THAN_2_CLASS.0));

    let decrement = crate::get(DECREMENT_ID);
    decrement.click();
    assert_eq!(button.class_name(), CLICKED_CLASS.0);
}
