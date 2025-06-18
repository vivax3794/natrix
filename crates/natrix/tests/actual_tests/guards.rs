use natrix::prelude::*;
use natrix::{guard_option, guard_result};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON: Id = natrix::id!();
const TEXT: Id = natrix::id!();

#[derive(Component)]
struct GuardTester {
    value: Option<u8>,
}

impl Component for GuardTester {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: E<Self>, _, _| match &mut *ctx.value {
                        Some(2) => *ctx.value = None,
                        Some(value) => *value += 1,
                        None => *ctx.value = Some(0),
                    }),
            )
            .child(|ctx: R<Self>| {
                if let Some(value_guard) = guard_option!(@owned |ctx| ctx.value) {
                    e::div().text(move |ctx: R<Self>| ctx.get_owned(&value_guard))
                } else {
                    e::div().text("NO VALUE")
                }
                .id(TEXT)
            })
    }
}

#[wasm_bindgen_test]
fn guard_works() {
    crate::mount_test(GuardTester { value: None });

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
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: E<Self>, _, _| match &mut *ctx.value {
                        Ok(value) => *value += 1,
                        Err(_) => *ctx.value = Ok(0),
                    }),
            )
            .child(|ctx: R<Self>| {
                match guard_result!(@owned |ctx| ctx.value) {
                    Ok(value_guard) => {
                        e::div().text(move |ctx: R<Self>| ctx.get_owned(&value_guard))
                    }
                    Err(error_guard) => {
                        e::div().text(move |ctx: R<Self>| ctx.get_owned(&error_guard))
                    }
                }
                .id(TEXT)
            })
    }
}

#[wasm_bindgen_test]
fn guard_result() {
    crate::mount_test(GuardTesterResult { value: Err(100) });

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
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: E<Self>, _, _| match &mut *ctx.value {
                        Some(Some(2)) => *ctx.value = None,
                        Some(Some(value)) => *value += 1,
                        Some(None) => *ctx.value = Some(Some(0)),
                        None => *ctx.value = Some(None),
                    }),
            )
            .child(|ctx: R<Self>| {
                if let Some(value_guard) = guard_option!(@owned |ctx| ctx.value) {
                    e::div().text(move |ctx: R<Self>| {
                        if let Some(inner_guard) =
                            guard_option!(@owned |ctx| ctx.get_owned(&value_guard))
                        {
                            e::div()
                                .id(TEXT)
                                .text(move |ctx: R<Self>| ctx.get_owned(&inner_guard))
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
    crate::mount_test(GuardTesterNested { value: None });

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
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: E<Self>, _, _| {
                        *ctx.value = *ctx.next;
                    }),
            )
            .child(|ctx: R<Self>| {
                if let Some(value_guard) = guard_option!(@owned |ctx| ctx.value) {
                    e::div().text(move |ctx: R<Self>| {
                        if let Some(inner_guard) =
                            guard_option!(@owned |ctx| ctx.get_owned(&value_guard))
                        {
                            e::div().id(TEXT).text(move |ctx: R<Self>| {
                                if ctx.get_owned(&inner_guard) {
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

struct NonCopy;

impl NonCopy {
    fn use_ref(&self) -> &'static str {
        "hello"
    }
}

#[derive(Component)]
struct NonCopyComponent {
    value: Option<NonCopy>,
}

impl Component for NonCopyComponent {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: E<Self>, _, _| {
                        *ctx.value = Some(NonCopy);
                    }),
            )
            .child(|ctx: R<Self>| {
                if let Some(value_guard) = guard_option!(|ctx| ctx.value.as_ref()) {
                    e::div()
                        .text(move |ctx: R<Self>| ctx.get(&value_guard).use_ref())
                        .id(TEXT)
                } else {
                    e::div().text("NO VALUE").id(TEXT)
                }
            })
    }
}

#[wasm_bindgen_test]
fn guard_non_copy() {
    crate::mount_test(NonCopyComponent { value: None });

    let button = crate::get(BUTTON);

    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("NO VALUE".to_owned()));

    button.click();
    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("hello".to_owned()));
}
