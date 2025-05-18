use natrix::prelude::*;
use natrix::reactivity::EventToken;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON_ID: &str = "BUTTON";

#[derive(Component)]
struct Counter {
    value: u8,
}

impl Component for Counter {
    type EmitMessage = u8;
    type ReceiveMessage = u8;
    fn render() -> impl Element<Self> {
        e::button()
            .id(BUTTON_ID)
            .text(|ctx: R<Self>| *ctx.value)
            .on::<events::Click>(|ctx: E<Self>, token, _| {
                *ctx.value += 1;
                ctx.emit(*ctx.value, token);
            })
    }

    fn handle_message(ctx: E<Self>, msg: Self::ReceiveMessage, token: EventToken) {
        *ctx.value += msg;
        ctx.emit(*ctx.value, token);
    }
}

const DOUBLE_ID: &str = "DOUBLE_ID";
const ADD_ID: &str = "ADD_ID";
const PARENT_ADD_ID: &str = "PARENT_ADD_ID";

#[derive(Component)]
struct RootOne {
    double: u8,
}

impl Component for RootOne {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        let (child, sender) = SubComponent::new(Counter { value: 0 }).sender();
        let sender_clone = sender.clone();

        e::div()
            .child(child.on(|ctx: E<Self>, amount, _| {
                *ctx.double = amount * 2;
            }))
            .child(e::div().id(DOUBLE_ID).text(|ctx: R<Self>| *ctx.double))
            .child(
                e::button()
                    .id(ADD_ID)
                    .on::<events::Click>(move |_ctx: E<Self>, token, _| {
                        sender.send(10, token);
                    }),
            )
            .child(
                e::button()
                    .id(PARENT_ADD_ID)
                    .on::<events::Click>(|ctx: E<Self>, token, _| {
                        *ctx.double += 10;
                    }),
            )
    }
}

#[wasm_bindgen_test]
fn simple_button_child() {
    crate::mount_test(RootOne { double: 0 });

    let button = crate::get(BUTTON_ID);

    assert_eq!(button.text_content(), Some("0".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("1".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("2".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("3".to_owned()));
}

#[wasm_bindgen_test]
async fn child_to_parent() {
    use natrix::async_utils;
    crate::mount_test(RootOne { double: 0 });

    let button = crate::get(BUTTON_ID);
    let double = crate::get(DOUBLE_ID);

    assert_eq!(button.text_content(), Some("0".to_owned()));
    assert_eq!(double.text_content(), Some("0".to_owned()));

    button.click();
    async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("1".to_owned()));
    assert_eq!(double.text_content(), Some("2".to_owned()));

    button.click();
    async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("2".to_owned()));
    assert_eq!(double.text_content(), Some("4".to_owned()));

    button.click();
    async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("3".to_owned()));
    assert_eq!(double.text_content(), Some("6".to_owned()));
}

#[wasm_bindgen_test]
async fn parent_to_child() {
    use natrix::async_utils;
    crate::mount_test(RootOne { double: 0 });

    let button = crate::get(BUTTON_ID);
    let double = crate::get(DOUBLE_ID);
    let add_button = crate::get(ADD_ID);

    assert_eq!(button.text_content(), Some("0".to_owned()));
    assert_eq!(double.text_content(), Some("0".to_owned()));

    add_button.click();
    assert_eq!(button.text_content(), Some("0".to_owned()));
    assert_eq!(double.text_content(), Some("0".to_owned()));

    async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("10".to_owned()));
    assert_eq!(double.text_content(), Some("20".to_owned()));

    add_button.click();
    async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("20".to_owned()));
    assert_eq!(double.text_content(), Some("40".to_owned()));

    add_button.click();
    async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("30".to_owned()));
    assert_eq!(double.text_content(), Some("60".to_owned()));
}

#[derive(Component)]
struct ChildTwo {
    value: u8,
}

impl Component for ChildTwo {
    type EmitMessage = NoMessages;
    type ReceiveMessage = u8;
    fn render() -> impl Element<Self> {
        e::div().id(BUTTON_ID).text(|ctx: R<Self>| *ctx.value)
    }

    fn handle_message(ctx: E<Self>, msg: Self::ReceiveMessage, _token: EventToken) {
        *ctx.value = msg;
    }
}

#[derive(Component)]
struct RootTwo {
    value: u8,
}

impl Component for RootTwo {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        |ctx: R<Self>| {
            let (child, sender) = SubComponent::new(ChildTwo { value: 0 }).sender();

            ctx.on_change(
                |ctx| {
                    *ctx.value;
                },
                move |ctx, token| {
                    sender.send(*ctx.value, token);
                },
            );

            e::div()
                .child(child)
                .child(
                    e::button()
                        .id(ADD_ID)
                        .on::<events::Click>(|ctx: E<Self>, _token, _| {
                            *ctx.value += 1;
                        }),
                )
        }
    }
}

#[wasm_bindgen_test]
async fn on_change() {
    crate::mount_test(RootTwo { value: 0 });

    let button = crate::get(BUTTON_ID);
    let add_button = crate::get(ADD_ID);

    assert_eq!(button.text_content(), Some("0".to_owned()));

    add_button.click();
    natrix::async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("1".to_owned()));

    add_button.click();
    natrix::async_utils::next_animation_frame().await;
    assert_eq!(button.text_content(), Some("2".to_owned()));
}
