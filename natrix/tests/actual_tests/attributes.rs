#![allow(dead_code)]

use natrix::html_elements::ToAttribute;
use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
wasm_bindgen_test_configure!(run_in_browser);

const ROOT: &str = "ROOT";

#[derive(Component, Default)]
struct Generic<T>(T);

impl<T: ToAttribute<Self::Data> + Copy> Component for Generic<T> {
    fn render() -> impl Element<Self::Data> {
        e::div().attr("abc", |ctx: R<Self>| *ctx.0).id(ROOT)
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
    fn render() -> impl Element<Self::Data> {
        e::button()
            .id(ROOT)
            .attr("abc", |ctx: R<Self>| format!("{}", *ctx.value))
            .on::<events::Click>(|ctx: &mut S<Self>, _| {
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
    fn render() -> impl Element<Self::Data> {
        e::button()
            .id(ROOT)
            .attr("abc", |ctx: R<Self>| *ctx.value)
            .on::<events::Click>(|ctx: &mut S<Self>, _| {
                *ctx.value = !*ctx.value;
            })
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
