use natrix::dom::ToAttribute;
use natrix::prelude::*;
use natrix::reactivity::NonReactive;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
wasm_bindgen_test_configure!(run_in_browser);

const ROOT: &str = "ROOT";
const BUTTON: &str = "BUTTON";

#[derive(Component, Default)]
struct Generic<T>(T);

impl<T: ToAttribute<()> + Copy> Component for Generic<T> {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .attr("abc", |ctx: R<Self>| NonReactive(*ctx.0))
            .id(ROOT)
    }
}

#[wasm_bindgen_test]
fn simple_true() {
    crate::mount_test(Generic(true));

    let button = crate::get(ROOT);
    assert_eq!(button.get_attribute("abc"), Some("".to_owned()));
}
#[wasm_bindgen_test]
fn simple_false() {
    crate::mount_test(Generic(false));

    let button = crate::get(ROOT);
    assert_eq!(button.get_attribute("abc"), None);
}
#[wasm_bindgen_test]
fn simple_string() {
    crate::mount_test(Generic("hello"));

    let button = crate::get(ROOT);
    assert_eq!(button.get_attribute("abc"), Some("hello".to_owned()));
}
#[wasm_bindgen_test]
fn simple_some() {
    crate::mount_test(Generic(Some("hello")));

    let button = crate::get(ROOT);
    assert_eq!(button.get_attribute("abc"), Some("hello".to_owned()));
}
#[wasm_bindgen_test]
fn simple_none() {
    crate::mount_test(Generic(None::<u8>));

    let button = crate::get(ROOT);
    assert_eq!(button.get_attribute("abc"), None);
}

#[wasm_bindgen_test]
fn simple_ok() {
    crate::mount_test(Generic(Ok::<&'static str, &'static str>("hello")));

    let button = crate::get(ROOT);
    assert_eq!(button.get_attribute("abc"), Some("hello".to_owned()));
}
#[wasm_bindgen_test]
fn simple_err() {
    crate::mount_test(Generic(Err::<&'static str, &'static str>("world")));

    let button = crate::get(ROOT);
    assert_eq!(button.get_attribute("abc"), Some("world".to_owned()));
}

#[derive(Component, Default)]
struct Counter {
    value: u8,
}

impl Component for Counter {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::button()
            .id(ROOT)
            .attr("abc", |ctx: R<Self>| format!("{}", *ctx.value))
            .on::<events::Click>(|ctx: E<Self>, _, _| {
                *ctx.value += 1;
            })
    }
}

#[wasm_bindgen_test]
fn reactive_attribute() {
    crate::mount_test(Counter::default());

    let button = crate::get(ROOT);

    assert_eq!(button.get_attribute("abc"), Some("0".to_owned()));

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("1".to_owned()));

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("2".to_owned()));

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("3".to_owned()));

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("4".to_owned()));
}

#[derive(Component, Default)]
struct Toggle {
    value: bool,
}

impl Component for Toggle {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;
    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::button()
                    .id(ROOT)
                    .attr("abc", |ctx: R<Self>| *ctx.value)
                    .on::<events::Click>(|ctx: E<Self>, _, _| {
                        *ctx.value = !*ctx.value;
                    }),
            )
            .child(
                e::button()
                    .id(BUTTON)
                    .on::<events::Click>(|ctx: E<Self>, _, _| {
                        *ctx.value = false;
                    }),
            )
    }
}

#[wasm_bindgen_test]
fn reactive_bool() {
    crate::mount_test(Toggle::default());

    let button = crate::get(ROOT);

    assert_eq!(button.get_attribute("abc"), None);

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("".to_owned()));

    button.click();
    assert_eq!(button.get_attribute("abc"), None);
}

#[wasm_bindgen_test]
fn reactive_change_set_but_no_change() {
    crate::mount_test(Toggle::default());

    let button = crate::get(ROOT);
    let button2 = crate::get(BUTTON);

    assert_eq!(button.get_attribute("abc"), None);

    button.click();
    assert_eq!(button.get_attribute("abc"), Some("".to_owned()));

    button2.click();
    assert_eq!(button.get_attribute("abc"), None);

    button2.click();
    assert_eq!(button.get_attribute("abc"), None);
}
