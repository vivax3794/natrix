use natrix::css_values::{Color, Numeric};
use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const ELEMENT_ID: &str = "ELEMENT_ID";

#[derive(Component)]
struct Comp {
    size: u8,
}

impl Component for Comp {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;

    fn render() -> impl Element<Self> {
        e::div()
            .id(ELEMENT_ID)
            .css_value("--rem", Numeric::rem(10))
            .css_value("--color", Color::rgb(255, 100, 10))
            .css_value("--size", |ctx: R<Self>| Numeric::rem(*ctx.size))
            .on::<events::Click>(|ctx: E<Self>, _| {
                *ctx.size += 1;
            })
    }
}

#[wasm_bindgen_test]
fn test_simple_unit() {
    crate::mount_test(Comp { size: 0 });

    let element = crate::get(ELEMENT_ID);

    let style = element.style();
    let value = style.get_property_value("--rem");

    assert_eq!(value, Ok("10rem".to_string()));
}

#[wasm_bindgen_test]
fn test_color() {
    crate::mount_test(Comp { size: 0 });

    let element = crate::get(ELEMENT_ID);

    let style = element.style();
    let value = style.get_property_value("--color");

    assert_eq!(value, Ok("rgb(255 100 10/1)".to_string()));
}

#[wasm_bindgen_test]
fn test_reactive() {
    crate::mount_test(Comp { size: 0 });

    let element = crate::get(ELEMENT_ID);
    let style = element.style();

    let value = style.get_property_value("--size");
    assert_eq!(value, Ok("0rem".to_string()));

    element.click();
    let value = style.get_property_value("--size");
    assert_eq!(value, Ok("1rem".to_string()));

    element.click();
    let value = style.get_property_value("--size");
    assert_eq!(value, Ok("2rem".to_string()));
}
