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

const DOUBLE_ID: Id = natrix::id!();
const ADD_ID: Id = natrix::id!();
const PARENT_ADD_ID: Id = natrix::id!();

#[derive(Component)]
struct RootOne {
    double: u8,
}

impl Component for RootOne {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        let child = SubComponent::new(Counter { value: 0 });
        let sender = child.sender();
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
fn child_to_parent() {
    crate::mount_test(RootOne { double: 0 });

    let button = crate::get(BUTTON_ID);
    let double = crate::get(DOUBLE_ID);

    assert_eq!(button.text_content(), Some("0".to_owned()));
    assert_eq!(double.text_content(), Some("0".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("1".to_owned()));
    assert_eq!(double.text_content(), Some("2".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("2".to_owned()));
    assert_eq!(double.text_content(), Some("4".to_owned()));

    button.click();
    assert_eq!(button.text_content(), Some("3".to_owned()));
    assert_eq!(double.text_content(), Some("6".to_owned()));
}

#[wasm_bindgen_test]
fn parent_to_child() {
    crate::mount_test(RootOne { double: 0 });

    let button = crate::get(BUTTON_ID);
    let double = crate::get(DOUBLE_ID);
    let add_button = crate::get(ADD_ID);

    assert_eq!(button.text_content(), Some("0".to_owned()));
    assert_eq!(double.text_content(), Some("0".to_owned()));

    add_button.click();
    assert_eq!(button.text_content(), Some("10".to_owned()));
    assert_eq!(double.text_content(), Some("20".to_owned()));

    add_button.click();
    assert_eq!(button.text_content(), Some("20".to_owned()));
    assert_eq!(double.text_content(), Some("40".to_owned()));

    add_button.click();
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

const RC_CHILD_ID: Id = natrix::id!();
const START_ID: Id = natrix::id!();
const RESULT_ID: Id = natrix::id!();

#[derive(Component)]
struct RecursiveChild {
    value: u8,
}

impl Component for RecursiveChild {
    type EmitMessage = u8;
    type ReceiveMessage = u8;

    fn render() -> impl Element<Self> {
        e::button().id(RC_CHILD_ID).text(|ctx: R<Self>| *ctx.value)
    }

    fn handle_message(ctx: E<Self>, msg: Self::ReceiveMessage, token: EventToken) {
        *ctx.value = msg;
        ctx.emit(msg, token);
    }
}

#[derive(Component)]
struct RootRecursive {
    max_rounds: u8,
    last: u8,
}

impl Component for RootRecursive {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;

    fn render() -> impl Element<Self> {
        let child = SubComponent::new(RecursiveChild { value: 0 });
        let sender = child.sender();
        let child_sender = child.sender();

        e::div()
            .child(child.on(move |ctx: E<Self>, msg, token| {
                *ctx.last = msg;
                if msg < *ctx.max_rounds {
                    child_sender.send(msg + 1, token);
                }
            }))
            .child(e::div().id(RESULT_ID).text(|ctx: R<Self>| *ctx.last))
            .child(e::button().id(START_ID).text("Start").on::<events::Click>(
                move |_ctx: E<Self>, token, _| {
                    sender.send(1, token);
                },
            ))
    }
}

#[wasm_bindgen_test]
fn recursive_message_test() {
    crate::mount_test(RootRecursive {
        max_rounds: 3,
        last: 0,
    });

    let start = crate::get(START_ID);
    let child = crate::get(RC_CHILD_ID);
    let result = crate::get(RESULT_ID);

    assert_eq!(child.text_content(), Some("0".to_owned()));
    assert_eq!(result.text_content(), Some("0".to_owned()));

    start.click();
    assert_eq!(child.text_content(), Some("3".to_owned()));
    assert_eq!(result.text_content(), Some("3".to_owned()));
}
