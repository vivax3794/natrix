use natrix::list::List;
use natrix::prelude::*;
use natrix::state::State;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

const ADD_BUTTON_ID: &str = "__ADD_BUTTON";
const REMOVE_BUTTON_ID: &str = "__REMOVE_BUTTON";
const CHANGE_BUTTON_ID: &str = "__CHANGE_BUTTON";

#[derive(Component, Default)]
struct ManualLoop {
    items: Vec<usize>,
}

impl Component for ManualLoop {
    type EmitMessage = NoMessages;
    type ReceiveMessage = NoMessages;

    fn render() -> impl Element<Self> {
        e::div()
            .child(
                e::button()
                    .id(ADD_BUTTON_ID)
                    .on::<events::Click>(|ctx: E<Self>, _| {
                        let len = ctx.items.len();
                        ctx.items.push(len);
                    }),
            )
            .child(
                e::button()
                    .id(REMOVE_BUTTON_ID)
                    .on::<events::Click>(|ctx: E<Self>, _| {
                        ctx.items.pop();
                    }),
            )
            .child(
                e::button()
                    .id(CHANGE_BUTTON_ID)
                    .on::<events::Click>(|ctx: E<Self>, _| {
                        ctx.items[0] = 100;
                    }),
            )
            .child(List::new(
                |ctx: &State<Self>| &ctx.items,
                |_ctx, getter| {
                    e::div()
                        .id(format!("item-parent-{}", getter.index))
                        .child(
                            e::div()
                                .text(move |ctx: R<Self>| getter.get_watched(ctx))
                                .id(format!("item-{}", getter.index)),
                        )
                        .child(move |ctx: R<Self>| {
                            e::div()
                                .text(getter.get_watched(ctx))
                                .text(ctx.watch(|ctx| ctx.items.len()))
                                .id(format!("item-2-{}", getter.index))
                        })
                },
            ))
    }
}

// These tests also demostate, by not refetching the elements, that the untouched elements are
// not rendered again.

#[wasm_bindgen_test]
fn add_works() {
    crate::mount_test(ManualLoop::default());

    let add_button = crate::get(ADD_BUTTON_ID);

    add_button.click();
    let item = crate::get("item-0");
    assert_eq!(item.text_content(), Some("0".to_owned()));

    add_button.click();
    let item2 = crate::get("item-1");
    assert_eq!(item.text_content(), Some("0".to_owned()));
    assert_eq!(item2.text_content(), Some("1".to_owned()));

    add_button.click();
    let item3 = crate::get("item-2");
    assert_eq!(item.text_content(), Some("0".to_owned()));
    assert_eq!(item2.text_content(), Some("1".to_owned()));
    assert_eq!(item3.text_content(), Some("2".to_owned()));
}

#[wasm_bindgen_test]
fn remove_works() {
    crate::mount_test(ManualLoop::default());

    let add_button = crate::get(ADD_BUTTON_ID);
    let remove_button = crate::get(REMOVE_BUTTON_ID);

    add_button.click();
    add_button.click();
    add_button.click();

    let item = crate::get("item-0");
    let item2 = crate::get("item-1");
    let item3 = crate::get("item-2");
    let item3_parent = crate::get("item-parent-2");

    assert_eq!(item.text_content(), Some("0".to_owned()));
    assert_eq!(item2.text_content(), Some("1".to_owned()));
    assert_eq!(item3.text_content(), Some("2".to_owned()));

    remove_button.click();
    assert_eq!(item.text_content(), Some("0".to_owned()));
    assert_eq!(item2.text_content(), Some("1".to_owned()));
    assert!(
        item3_parent.parent_node().is_none(),
        "Item 3 should be removed"
    );
}

#[wasm_bindgen_test]
fn change_works() {
    crate::mount_test(ManualLoop::default());

    let add_button = crate::get(ADD_BUTTON_ID);
    let change_button = crate::get(CHANGE_BUTTON_ID);

    add_button.click();
    add_button.click();
    add_button.click();

    let item = crate::get("item-0");
    assert_eq!(item.text_content(), Some("0".to_owned()));

    change_button.click();
    assert_eq!(item.text_content(), Some("100".to_owned()));
}

#[wasm_bindgen_test]
fn change_only_triggers_actual_changed() {
    crate::mount_test(ManualLoop::default());

    let add_button = crate::get(ADD_BUTTON_ID);
    let change_button = crate::get(CHANGE_BUTTON_ID);

    add_button.click();
    add_button.click();
    add_button.click();

    let item = crate::get("item-2-1");
    change_button.click();
    assert!(item.parent_node().is_some(), "unneeded re-rendered");
}
