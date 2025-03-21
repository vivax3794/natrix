#![allow(dead_code)]

use natrix::prelude::*;
use proptest::proptest;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON: &str = "__BUTTON";
const TEXT: &str = "__TEXT";

#[derive(Component)]
struct GuardTester {
    value: Option<u8>,
}

impl Component for GuardTester {
    fn render() -> impl Element<Self::Data> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: &mut S<Self>, _| match &mut *ctx.value {
                        Some(2) => *ctx.value = None,
                        Some(value) => *value += 1,
                        None => *ctx.value = Some(0),
                    }),
            )
            .child(|mut ctx: R<Self>| {
                if let Some(value_guard) = guard_option!(ctx.value) {
                    e::div().text(move |ctx: R<Self>| ctx.get(&value_guard))
                } else {
                    e::div().text("NO VALUE")
                }
                .id(TEXT)
            })
    }
}

#[wasm_bindgen_test]
fn guard_works() {
    crate::setup();
    mount_component(GuardTester { value: None }, crate::MOUNT_POINT);

    let button = crate::get(BUTTON);

    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("NO VALUE".to_owned()));

    button.click();
    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("0".to_owned()));

    button.click();
    assert_eq!(text.text_content(), Some("1".to_owned()));

    button.click();
    assert_eq!(text.text_content(), Some("2".to_owned()));

    button.click();
    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("NO VALUE".to_owned()));
}

#[derive(Component)]
struct GuardTesterResult {
    value: Result<u8, u8>,
}

impl Component for GuardTesterResult {
    fn render() -> impl Element<Self::Data> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: &mut S<Self>, _| match &mut *ctx.value {
                        Ok(value) => *value += 1,
                        Err(_) => *ctx.value = Ok(0),
                    }),
            )
            .child(|mut ctx: R<Self>| {
                match guard_result!(ctx.value) {
                    Ok(value_guard) => e::div().text(move |ctx: R<Self>| ctx.get(&value_guard)),
                    Err(error_guard) => e::div().text(move |ctx: R<Self>| ctx.get(&error_guard)),
                }
                .id(TEXT)
            })
    }
}

#[wasm_bindgen_test]
fn guard_result() {
    crate::setup();
    mount_component(GuardTesterResult { value: Err(100) }, crate::MOUNT_POINT);

    let button = crate::get(BUTTON);

    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("100".to_owned()));

    button.click();
    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("0".to_owned()));

    button.click();
    assert_eq!(text.text_content(), Some("1".to_owned()));

    button.click();
    assert_eq!(text.text_content(), Some("2".to_owned()));
}

#[derive(Component)]
struct GuardTesterNested {
    value: Option<Option<u8>>,
}

impl Component for GuardTesterNested {
    fn render() -> impl Element<Self::Data> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: &mut S<Self>, _| match &mut *ctx.value {
                        Some(Some(2)) => *ctx.value = None,
                        Some(Some(value)) => *value += 1,
                        Some(None) => *ctx.value = Some(Some(0)),
                        None => *ctx.value = Some(None),
                    }),
            )
            .child(|mut ctx: R<Self>| {
                if let Some(value_guard) = guard_option!(ctx.value) {
                    e::div().text(move |mut ctx: R<Self>| {
                        if let Some(inner_guard) = guard_option!(ctx.get(&value_guard)) {
                            e::div()
                                .id(TEXT)
                                .text(move |ctx: R<Self>| ctx.get(&inner_guard))
                        } else {
                            e::div().text("NO VALUE INNER").id(TEXT)
                        }
                    })
                } else {
                    e::div().text("NO VALUE").id(TEXT)
                }
            })
    }
}

#[wasm_bindgen_test]
fn guard_nested() {
    crate::setup();
    mount_component(GuardTesterNested { value: None }, crate::MOUNT_POINT);

    let button = crate::get(BUTTON);

    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("NO VALUE".to_owned()));

    button.click();
    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("NO VALUE INNER".to_owned()));

    button.click();
    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("0".to_owned()));

    button.click();
    assert_eq!(text.text_content(), Some("1".to_owned()));

    button.click();
    assert_eq!(text.text_content(), Some("2".to_owned()));

    button.click();
    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("NO VALUE".to_owned()));
}

#[derive(Component)]
struct GuardSwitchProp {
    value: Option<Option<bool>>,
    next: Option<Option<bool>>,
}

impl Component for GuardSwitchProp {
    fn render() -> impl Element<Self::Data> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: &mut S<Self>, _| {
                        *ctx.value = *ctx.next;
                    }),
            )
            .child(|mut ctx: R<Self>| {
                if let Some(value_guard) = guard_option!(ctx.value) {
                    e::div().text(move |mut ctx: R<Self>| {
                        if let Some(inner_guard) = guard_option!(ctx.get(&value_guard)) {
                            e::div().id(TEXT).text(move |ctx: R<Self>| {
                                if ctx.get(&inner_guard) {
                                    "hello"
                                } else {
                                    "world"
                                }
                            })
                        } else {
                            e::div().text("NO VALUE INNER").id(TEXT)
                        }
                    })
                } else {
                    e::div().text("NO VALUE").id(TEXT)
                }
            })
    }
}

proptest! {
    #[wasm_bindgen_test]
    fn guard_switch(start: Option<Option<bool>>, next: Option<Option<bool>>) {
        crate::setup();
        mount_component(GuardSwitchProp {value: start, next}, crate::MOUNT_POINT);

        let button = crate::get(BUTTON);
        button.click();
    }
}
