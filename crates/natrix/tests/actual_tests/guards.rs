use natrix::prelude::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const BUTTON: Id = natrix::id!();
const TEXT: Id = natrix::id!();

#[derive(State)]
struct TestOption {
    value: Signal<Option<u8>>,
}

fn render_test_option() -> impl Element<TestOption> {
    e::div()
        .child(
            e::button()
                .id(BUTTON)
                .on::<events::Click>(|ctx: &mut Ctx<TestOption>, _, _| match &mut *ctx.value {
                    Some(2) => *ctx.value = None,
                    Some(value) => *value += 1,
                    None => *ctx.value = Some(0),
                }),
        )
        .child(|ctx: &mut RenderCtx<TestOption>| {
            if let Some(value_guard) = ctx.guard(lens!(TestOption => .value).deref()) {
                e::div().text(move |ctx: &mut RenderCtx<TestOption>| *ctx.get(value_guard))
            } else {
                e::div().text("NO VALUE")
            }
            .id(TEXT)
        })
}

#[wasm_bindgen_test]
fn guard_works() {
    crate::mount_test(
        TestOption {
            value: Signal::new(None),
        },
        render_test_option(),
    );

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

#[derive(State)]
struct TestResult {
    value: Signal<Result<u8, u8>>,
}

fn render_test_result() -> impl Element<TestResult> {
    e::div()
        .child(
            e::button()
                .id(BUTTON)
                .on::<events::Click>(|ctx: &mut Ctx<TestResult>, _, _| match &mut *ctx.value {
                    Ok(value) => *value += 1,
                    Err(_) => *ctx.value = Ok(0),
                }),
        )
        .child(|ctx: &mut RenderCtx<TestResult>| {
            match ctx.guard(lens!(TestResult => .value).deref()) {
                Ok(value_guard) => {
                    e::div().text(move |ctx: &mut RenderCtx<TestResult>| *ctx.get(value_guard))
                }
                Err(error_guard) => {
                    e::div().text(move |ctx: &mut RenderCtx<TestResult>| *ctx.get(error_guard))
                }
            }
            .id(TEXT)
        })
}

#[wasm_bindgen_test]
fn guard_result() {
    crate::mount_test(
        TestResult {
            value: Signal::new(Err(100)),
        },
        render_test_result(),
    );

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

#[derive(State)]
struct Nested {
    value: Signal<Option<Option<u8>>>,
}

fn render_nested() -> impl Element<Nested> {
    e::div()
        .child(
            e::button().id(BUTTON).on::<events::Click>(
                |ctx: &mut Ctx<Nested>, _, _| match &mut *ctx.value {
                    Some(Some(2)) => *ctx.value = None,
                    Some(Some(value)) => *value += 1,
                    Some(None) => *ctx.value = Some(Some(0)),
                    None => *ctx.value = Some(None),
                },
            ),
        )
        .child(|ctx: &mut RenderCtx<Nested>| {
            if let Some(value_guard) = ctx.guard(lens!(Nested => .value).deref()) {
                e::div().text(move |ctx: &mut RenderCtx<Nested>| {
                    if let Some(inner_guard) = ctx.guard(value_guard) {
                        e::div()
                            .id(TEXT)
                            .text(move |ctx: &mut RenderCtx<Nested>| *ctx.get(inner_guard))
                    } else {
                        e::div().text("NO VALUE INNER").id(TEXT)
                    }
                })
            } else {
                e::div().text("NO VALUE").id(TEXT)
            }
        })
}

#[wasm_bindgen_test]
fn guard_nested() {
    crate::mount_test(
        Nested {
            value: Signal::new(None),
        },
        render_nested(),
    );

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

// NOTE: This tests for a re-ordering bug in the reactive runtime, where the internal ordering of
// signal dependencies generally covered up that the update cycle was reading hooks in the wrong
// order.
// This test is specically setup to have the first read hook be in the wrong order.
// Cruically the order of the fields in this definition is crucial for triggering the bug
#[derive(State)]
struct ReactiveOrderingEdgeCaseRegression {
    trigger: Signal<u8>,
    guarded_value: Signal<Option<u8>>,
}

fn render_edge_case() -> impl Element<ReactiveOrderingEdgeCaseRegression> {
    e::div().child(
        e::button()
            .id(BUTTON)
            .on::<events::Click>(|ctx: &mut Ctx<ReactiveOrderingEdgeCaseRegression>, _, _| {
                *ctx.trigger += 1;
                if ctx.guarded_value.is_some() {
                    *ctx.guarded_value = None;
                } else {
                    *ctx.guarded_value = Some(0);
                }
            })
            .child(|ctx: &mut RenderCtx<ReactiveOrderingEdgeCaseRegression>| {
                if let Some(guard) =
                    ctx.guard(lens!(ReactiveOrderingEdgeCaseRegression => .guarded_value).deref())
                {
                    Some(e::div().id(TEXT).text(
                        move |ctx: &mut RenderCtx<ReactiveOrderingEdgeCaseRegression>| {
                            let trigger = *ctx.trigger;
                            format!("{}-{}", trigger, *ctx.get(guard))
                        },
                    ))
                } else {
                    None
                }
            }),
    )
}

#[wasm_bindgen_test]
fn reactive_ordering_edge_case_regression() {
    crate::mount_test(
        ReactiveOrderingEdgeCaseRegression {
            trigger: Signal::new(0),
            guarded_value: Signal::new(None),
        },
        render_edge_case(),
    );

    let button = crate::get(BUTTON);

    button.click();
    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("1-0".to_string()));

    button.click();
    button.click();
    let text = crate::get(TEXT);
    assert_eq!(text.text_content(), Some("3-0".to_string()));
}
